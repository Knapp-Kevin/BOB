//! Vendor CLIs (`claude -p`, `codex exec`) run as bounded external processes.
//!
//! Assist mode grants a CLI no tool, shell, filesystem, or repository authority:
//! Claude Code runs with every built-in tool disabled and customizations (hooks,
//! MCP, plugins, CLAUDE.md) off; Codex runs in its read-only sandbox, and B.O.B.
//! terminates the process the moment it reports any tool, command, file, MCP, or
//! web-search item. Both start in an empty scratch directory, receive the prompt
//! on stdin (never as an argument), and are killed with their process tree on
//! timeout or cancellation. Raw CLI output never reaches the frontend: errors are
//! normalized and only validated reply text is returned.

use serde::Serialize;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const GENERATION_TIMEOUT: Duration = Duration::from_secs(120);
const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_STDOUT_BYTES: usize = 256_000;
const MAX_STDERR_BYTES: usize = 16_000;
// Must match the canonical chat-entry persistence limit.
const MAX_GENERATION_BYTES: usize = 8_000;
const SYSTEM_PROMPT: &str = "You are B.O.B., a calm planning assistant. Reply in short plain text. You have no tools and cannot read files, run commands, or browse.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliKind {
    Claude,
    Codex,
}

impl CliKind {
    fn executable_names(self) -> &'static [&'static str] {
        match self {
            Self::Claude if cfg!(windows) => &["claude.exe", "claude.cmd"],
            Self::Codex if cfg!(windows) => &["codex.exe", "codex.cmd"],
            Self::Claude => &["claude"],
            Self::Codex => &["codex"],
        }
    }

    /// Fixed argument vector. The model ID is validated by the inference policy
    /// (no leading `-`, conservative charset) before it reaches this point.
    pub fn generation_args(self, model: &str) -> Vec<String> {
        let args: &[&str] = match self {
            Self::Claude => &[
                "-p",
                "--output-format",
                "json",
                "--model",
                model,
                "--tools",
                "",
                "--safe-mode",
                "--strict-mcp-config",
                "--no-session-persistence",
                "--permission-mode",
                "dontAsk",
                "--system-prompt",
                SYSTEM_PROMPT,
            ],
            Self::Codex => &[
                "exec",
                "--sandbox",
                "read-only",
                "--skip-git-repo-check",
                "--ephemeral",
                "--ignore-user-config",
                "--ignore-rules",
                "--json",
                "--color",
                "never",
                "-m",
                model,
                "-",
            ],
        };
        args.iter().map(|arg| (*arg).to_owned()).collect()
    }

    fn auth_status_args(self) -> &'static [&'static str] {
        match self {
            Self::Claude => &["auth", "status", "--json"],
            Self::Codex => &["login", "status"],
        }
    }
}

/// Normalized CLI failure classes. Display strings carry no CLI output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliFailure {
    NotInstalled,
    Unauthenticated,
    Timeout,
    Cancelled,
    InvalidOutput,
    ToolUseAttempted,
    ExecutionFailed,
}

impl std::fmt::Display for CliFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NotInstalled => "CLI runtime not installed",
            Self::Unauthenticated => "CLI runtime is not signed in",
            Self::Timeout => "CLI runtime timed out",
            Self::Cancelled => "CLI generation cancelled",
            Self::InvalidOutput => "CLI runtime returned invalid output",
            Self::ToolUseAttempted => "CLI runtime attempted tool use and was stopped",
            Self::ExecutionFailed => "CLI runtime failed",
        })
    }
}

impl std::error::Error for CliFailure {}

/// Shared cancellation flag for one in-flight generation.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

/// Finds the vendor executable on PATH. Never runs anything from the current
/// directory or a caller-supplied path.
pub fn locate(kind: CliKind) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .filter(|dir| dir.is_absolute())
        .flat_map(|dir| {
            kind.executable_names()
                .iter()
                .map(move |name| dir.join(name))
        })
        .find(|candidate| candidate.is_file())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthState {
    Ready,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub auth_state: AuthState,
    /// `account_session`, `api_key`, or `unknown`. Never an email or token.
    pub auth_method: Option<String>,
    /// Plan tier reported by the CLI (for example `max`), when it reports one.
    pub plan: Option<String>,
}

/// Version and sign-in detection. Runs only `--version` and the vendor's
/// auth-status command; never sends a prompt.
pub fn detect(kind: CliKind, program: Option<&Path>) -> CliStatus {
    let Some(program) = program else {
        return CliStatus {
            installed: false,
            version: None,
            auth_state: AuthState::Unknown,
            auth_method: None,
            plan: None,
        };
    };
    let version = run_process(
        program,
        &["--version".to_owned()],
        None,
        PROBE_TIMEOUT,
        &CancelToken::default(),
        |_| true,
    )
    .ok()
    .and_then(|output| parse_version(&output.stdout));
    let args: Vec<String> = kind
        .auth_status_args()
        .iter()
        .map(|arg| (*arg).to_owned())
        .collect();
    let auth = run_process(
        program,
        &args,
        None,
        PROBE_TIMEOUT,
        &CancelToken::default(),
        |_| true,
    );
    let (auth_state, auth_method, plan) = match (kind, auth) {
        (CliKind::Claude, Ok(output)) => parse_claude_auth(&output.stdout),
        (CliKind::Codex, Ok(output)) => parse_codex_auth(&output),
        (_, Err(_)) => (AuthState::Unknown, None, None),
    };
    CliStatus {
        installed: true,
        version,
        auth_state,
        auth_method,
        plan,
    }
}

fn parse_version(stdout: &str) -> Option<String> {
    stdout
        .split_whitespace()
        .find(|token| token.chars().next().is_some_and(|ch| ch.is_ascii_digit()))
        .filter(|token| {
            token.len() <= 32
                && token
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '+'))
        })
        .map(str::to_owned)
}

type AuthReport = (AuthState, Option<String>, Option<String>);

fn parse_claude_auth(stdout: &str) -> AuthReport {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(stdout.trim()) else {
        return (AuthState::Unknown, None, None);
    };
    if value["loggedIn"].as_bool() != Some(true) {
        return (AuthState::Missing, None, None);
    }
    let method = match value["authMethod"].as_str() {
        Some("claude.ai") => "account_session",
        Some(method) if method.to_ascii_lowercase().contains("key") => "api_key",
        _ => "unknown",
    };
    let plan = value["subscriptionType"]
        .as_str()
        .filter(|plan| plan.len() <= 32 && plan.chars().all(|ch| ch.is_ascii_alphanumeric()))
        .map(str::to_owned);
    (AuthState::Ready, Some(method.to_owned()), plan)
}

fn parse_codex_auth(output: &ProcessOutput) -> AuthReport {
    // `codex login status` may print to either stream; the API-key form shows a
    // masked key, so the raw text is classified here and never returned.
    let text = format!("{}\n{}", output.stdout, output.stderr).to_ascii_lowercase();
    if text.contains("not logged in") {
        return (AuthState::Missing, None, None);
    }
    if !output.success || !text.contains("logged in") {
        return (AuthState::Unknown, None, None);
    }
    let method = if text.contains("chatgpt") {
        "account_session"
    } else if text.contains("api key") {
        "api_key"
    } else {
        "unknown"
    };
    (AuthState::Ready, Some(method.to_owned()), None)
}

/// Runs one generation. `prompt` must already be bounded by the caller.
pub fn generate(
    kind: CliKind,
    program: &Path,
    model: &str,
    prompt: &str,
    timeout: Duration,
    cancel: &CancelToken,
) -> Result<String, CliFailure> {
    let scratch = ScratchDir::create().map_err(|_| CliFailure::ExecutionFailed)?;
    let args = kind.generation_args(model);
    let output = run_process_in(
        program,
        &args,
        Some(prompt.as_bytes()),
        timeout,
        cancel,
        Some(&scratch.0),
        |line| kind != CliKind::Codex || !codex_line_uses_tools(line),
    )?;
    let reply = match kind {
        CliKind::Claude => parse_claude_output(&output),
        CliKind::Codex => parse_codex_output(&output),
    }?;
    if reply.trim().is_empty() || reply.len() > MAX_GENERATION_BYTES {
        return Err(CliFailure::InvalidOutput);
    }
    Ok(reply)
}

fn mentions_auth_problem(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    [
        "not logged in",
        "/login",
        "log in",
        "login",
        "unauthorized",
        "authenticat",
        "invalid api key",
        "401",
    ]
    .iter()
    .any(|hint| text.contains(hint))
}

fn parse_claude_output(output: &ProcessOutput) -> Result<String, CliFailure> {
    let parsed = serde_json::from_str::<serde_json::Value>(output.stdout.trim());
    let Ok(value) = parsed else {
        return Err(
            if !output.success && mentions_auth_problem(&output.stderr) {
                CliFailure::Unauthenticated
            } else if output.success {
                CliFailure::InvalidOutput
            } else {
                CliFailure::ExecutionFailed
            },
        );
    };
    let result = value["result"].as_str().unwrap_or_default();
    if value["is_error"].as_bool() != Some(false)
        || value["subtype"].as_str() != Some("success")
        || !output.success
    {
        return Err(
            if mentions_auth_problem(result) || mentions_auth_problem(&output.stderr) {
                CliFailure::Unauthenticated
            } else {
                CliFailure::ExecutionFailed
            },
        );
    }
    if value["type"].as_str() != Some("result") {
        return Err(CliFailure::InvalidOutput);
    }
    Ok(result.to_owned())
}

const CODEX_TOOL_ITEMS: [&str; 5] = [
    "command_execution",
    "file_change",
    "mcp_tool_call",
    "web_search",
    "collab_tool_call",
];

fn codex_line_uses_tools(line: &str) -> bool {
    let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
        return false;
    };
    event["type"]
        .as_str()
        .is_some_and(|kind| kind.starts_with("item."))
        && event["item"]["type"]
            .as_str()
            .is_some_and(|item| CODEX_TOOL_ITEMS.contains(&item))
}

fn parse_codex_output(output: &ProcessOutput) -> Result<String, CliFailure> {
    let mut reply = None;
    let mut completed = false;
    let mut failure: Option<String> = None;
    for line in output.stdout.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        match event["type"].as_str() {
            Some("item.completed") if event["item"]["type"] == "agent_message" => {
                reply = event["item"]["text"].as_str().map(str::to_owned);
            }
            Some("turn.completed") => completed = true,
            Some("turn.failed") => {
                failure = Some(event["error"]["message"].as_str().unwrap_or("").to_owned())
            }
            Some("error") => failure = Some(event["message"].as_str().unwrap_or("").to_owned()),
            _ => {}
        }
    }
    if let Some(message) = failure {
        return Err(
            if mentions_auth_problem(&message) || mentions_auth_problem(&output.stderr) {
                CliFailure::Unauthenticated
            } else {
                CliFailure::ExecutionFailed
            },
        );
    }
    if !output.success {
        return Err(if mentions_auth_problem(&output.stderr) {
            CliFailure::Unauthenticated
        } else {
            CliFailure::ExecutionFailed
        });
    }
    match (completed, reply) {
        (true, Some(reply)) => Ok(reply),
        _ => Err(CliFailure::InvalidOutput),
    }
}

#[derive(Debug)]
struct ProcessOutput {
    stdout: String,
    stderr: String,
    success: bool,
}

/// Empty per-invocation working directory, removed afterwards.
struct ScratchDir(PathBuf);

impl ScratchDir {
    fn create() -> std::io::Result<Self> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!("bob-cli-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&path)?;
        Ok(Self(path))
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run_process(
    program: &Path,
    args: &[String],
    stdin: Option<&[u8]>,
    timeout: Duration,
    cancel: &CancelToken,
    accept_line: impl FnMut(&str) -> bool,
) -> Result<ProcessOutput, CliFailure> {
    run_process_in(program, args, stdin, timeout, cancel, None, accept_line)
}

/// Spawns `program`, streams stdout line by line through `accept_line` (which
/// may veto, killing the process), and enforces size, time, and cancellation
/// bounds. The whole process tree is killed on any early stop.
fn run_process_in(
    program: &Path,
    args: &[String],
    stdin: Option<&[u8]>,
    timeout: Duration,
    cancel: &CancelToken,
    cwd: Option<&Path>,
    mut accept_line: impl FnMut(&str) -> bool,
) -> Result<ProcessOutput, CliFailure> {
    let mut command = Command::new(program);
    command
        .args(args)
        .env("NO_COLOR", "1")
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            CliFailure::NotInstalled
        } else {
            CliFailure::ExecutionFailed
        }
    })?;

    if let (Some(input), Some(mut pipe)) = (stdin, child.stdin.take()) {
        let input = input.to_vec();
        std::thread::spawn(move || {
            let _ = pipe.write_all(&input);
        });
    }
    let stdout = child.stdout.take().ok_or(CliFailure::ExecutionFailed)?;
    let stderr = child.stderr.take().ok_or(CliFailure::ExecutionFailed)?;
    let (line_tx, line_rx) = mpsc::channel::<Result<String, ()>>();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut total = 0usize;
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => return,
                Ok(read) => {
                    total += read;
                    if total > MAX_STDOUT_BYTES {
                        let _ = line_tx.send(Err(()));
                        return;
                    }
                    if line_tx.send(Ok(line)).is_err() {
                        return;
                    }
                }
                Err(_) => {
                    let _ = line_tx.send(Err(()));
                    return;
                }
            }
        }
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = stderr
            .take(MAX_STDERR_BYTES as u64)
            .read_to_end(&mut buffer);
        String::from_utf8_lossy(&buffer).into_owned()
    });

    let deadline = Instant::now() + timeout;
    let mut collected = String::new();
    let mut stdout_open = true;
    let stop = |child: &mut Child, failure| {
        kill_tree(child);
        Err(failure)
    };
    loop {
        if cancel.is_cancelled() {
            return stop(&mut child, CliFailure::Cancelled);
        }
        if Instant::now() >= deadline {
            return stop(&mut child, CliFailure::Timeout);
        }
        if stdout_open {
            match line_rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok(line)) => {
                    if !accept_line(line.trim_end()) {
                        return stop(&mut child, CliFailure::ToolUseAttempted);
                    }
                    collected.push_str(&line);
                    continue;
                }
                Ok(Err(())) => return stop(&mut child, CliFailure::InvalidOutput),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => stdout_open = false,
            }
        } else {
            std::thread::sleep(Duration::from_millis(20));
        }
        if !stdout_open {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let stderr = stderr_reader.join().unwrap_or_default();
                    return Ok(ProcessOutput {
                        stdout: collected,
                        stderr,
                        success: status.success(),
                    });
                }
                Ok(None) => {}
                Err(_) => return stop(&mut child, CliFailure::ExecutionFailed),
            }
        }
    }
}

fn kill_tree(child: &mut Child) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        // `.cmd` shims start cmd.exe, which starts the real runtime; kill the tree.
        let _ = Command::new("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
pub(crate) mod fake {
    //! A fake vendor CLI: a script that records its arguments and stdin, then
    //! prints a canned response, optionally after sleeping.
    use std::path::{Path, PathBuf};

    pub struct FakeCli {
        pub dir: tempfile::TempDir,
        pub program: PathBuf,
    }

    impl FakeCli {
        pub fn invoked(&self) -> bool {
            self.dir.path().join("args.txt").exists()
        }
        pub fn args(&self) -> String {
            std::fs::read_to_string(self.dir.path().join("args.txt")).unwrap_or_default()
        }
        pub fn stdin(&self) -> String {
            std::fs::read_to_string(self.dir.path().join("stdin.txt")).unwrap_or_default()
        }
    }

    pub fn install(stdout: &str, exit_code: i32, sleep_seconds: u32) -> FakeCli {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("out.txt"), stdout).unwrap();
        let program = write_script(dir.path(), exit_code, sleep_seconds);
        FakeCli { dir, program }
    }

    #[cfg(windows)]
    fn write_script(dir: &Path, exit_code: i32, sleep_seconds: u32) -> PathBuf {
        let d = dir.display();
        let sleep = if sleep_seconds > 0 {
            format!("ping -n {} 127.0.0.1 > nul\r\n", sleep_seconds + 1)
        } else {
            String::new()
        };
        let script = format!(
            "@echo off\r\necho %* > \"{d}\\args.txt\"\r\nmore > \"{d}\\stdin.txt\"\r\n{sleep}type \"{d}\\out.txt\"\r\nexit /b {exit_code}\r\n"
        );
        let program = dir.join("fake-cli.cmd");
        std::fs::write(&program, script).unwrap();
        program
    }

    #[cfg(not(windows))]
    fn write_script(dir: &Path, exit_code: i32, sleep_seconds: u32) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let d = dir.display();
        let script = format!(
            "#!/bin/sh\necho \"$@\" > \"{d}/args.txt\"\ncat > \"{d}/stdin.txt\"\nsleep {sleep_seconds}\ncat \"{d}/out.txt\"\nexit {exit_code}\n"
        );
        let program = dir.join("fake-cli");
        std::fs::write(&program, script).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        program
    }
}

#[cfg(test)]
mod tests {
    use super::fake::install;
    use super::*;

    const PROMPT: &str = "Help me pick one next step";

    fn claude_success(text: &str) -> String {
        serde_json::json!({
            "type": "result", "subtype": "success", "is_error": false,
            "result": text, "num_turns": 1
        })
        .to_string()
    }

    fn run(kind: CliKind, fake: &fake::FakeCli, timeout: Duration) -> Result<String, CliFailure> {
        generate(
            kind,
            &fake.program,
            "test-model",
            PROMPT,
            timeout,
            &CancelToken::default(),
        )
    }

    #[test]
    fn claude_success_passes_prompt_on_stdin_with_tools_disabled() {
        let fake = install(&claude_success("Start with the email."), 0, 0);
        let reply = run(CliKind::Claude, &fake, Duration::from_secs(20)).unwrap();
        assert_eq!(reply, "Start with the email.");
        let args = fake.args();
        assert!(args.contains("-p"));
        assert!(args.contains("--tools"));
        assert!(args.contains("--safe-mode"));
        assert!(args.contains("--strict-mcp-config"));
        assert!(args.contains("--no-session-persistence"));
        assert!(args.contains("--model test-model"));
        assert!(!args.contains(PROMPT), "prompt must not be an argument");
        assert!(fake.stdin().contains(PROMPT));
    }

    #[test]
    fn codex_success_uses_read_only_sandbox_and_returns_final_message() {
        let events = [
            r#"{"type":"thread.started","thread_id":"t"}"#,
            r#"{"type":"turn.started"}"#,
            r#"{"type":"item.completed","item":{"id":"i0","type":"reasoning","text":"thinking"}}"#,
            r#"{"type":"item.completed","item":{"id":"i1","type":"agent_message","text":"Do the smallest piece."}}"#,
            r#"{"type":"turn.completed","usage":{}}"#,
        ]
        .join("\n");
        let fake = install(&events, 0, 0);
        let reply = run(CliKind::Codex, &fake, Duration::from_secs(20)).unwrap();
        assert_eq!(reply, "Do the smallest piece.");
        let args = fake.args();
        assert!(args.contains("exec"));
        assert!(args.contains("--sandbox read-only"));
        assert!(args.contains("--ephemeral"));
        assert!(args.contains("--json"));
        assert!(!args.contains(PROMPT));
        assert!(fake.stdin().contains(PROMPT));
    }

    #[test]
    fn auth_failures_are_classified() {
        let not_logged_in = serde_json::json!({
            "type": "result", "subtype": "success", "is_error": true,
            "result": "Not logged in · Please run /login"
        })
        .to_string();
        let fake = install(&not_logged_in, 1, 0);
        assert_eq!(
            run(CliKind::Claude, &fake, Duration::from_secs(20)),
            Err(CliFailure::Unauthenticated)
        );
        let codex =
            r#"{"type":"turn.failed","error":{"message":"401 Unauthorized: please login"}}"#;
        let fake = install(codex, 1, 0);
        assert_eq!(
            run(CliKind::Codex, &fake, Duration::from_secs(20)),
            Err(CliFailure::Unauthenticated)
        );
    }

    #[test]
    fn malformed_or_empty_output_is_rejected() {
        let cases = [
            (CliKind::Claude, "not json".to_owned(), 0),
            (CliKind::Claude, "{}".to_owned(), 0),
            (CliKind::Claude, claude_success("   "), 0),
            (
                CliKind::Claude,
                claude_success(&"x".repeat(MAX_GENERATION_BYTES + 1)),
                0,
            ),
            (CliKind::Claude, "boom".to_owned(), 3),
            (CliKind::Codex, "".to_owned(), 0),
            (
                CliKind::Codex,
                r#"{"type":"item.completed","item":{"type":"agent_message","text":"partial"}}"#
                    .to_owned(),
                0,
            ),
            (
                CliKind::Codex,
                r#"{"type":"error","message":"model overloaded"}"#.to_owned(),
                1,
            ),
        ];
        for (kind, stdout, code) in cases {
            let fake = install(&stdout, code, 0);
            let result = run(kind, &fake, Duration::from_secs(20));
            assert!(
                matches!(
                    result,
                    Err(CliFailure::InvalidOutput) | Err(CliFailure::ExecutionFailed)
                ),
                "{kind:?} {stdout}: {result:?}"
            );
        }
    }

    #[test]
    fn codex_tool_use_is_stopped() {
        let events = [
            r#"{"type":"turn.started"}"#,
            r#"{"type":"item.started","item":{"id":"i1","type":"command_execution","command":"cat secrets"}}"#,
            r#"{"type":"item.completed","item":{"id":"i2","type":"agent_message","text":"done"}}"#,
            r#"{"type":"turn.completed"}"#,
        ]
        .join("\n");
        let fake = install(&events, 0, 0);
        assert_eq!(
            run(CliKind::Codex, &fake, Duration::from_secs(20)),
            Err(CliFailure::ToolUseAttempted)
        );
    }

    #[test]
    fn timeout_kills_the_process_tree() {
        let fake = install(&claude_success("late"), 0, 30);
        let started = Instant::now();
        assert_eq!(
            run(CliKind::Claude, &fake, Duration::from_secs(2)),
            Err(CliFailure::Timeout)
        );
        assert!(started.elapsed() < Duration::from_secs(15));
    }

    #[test]
    fn cancellation_stops_generation() {
        let fake = install(&claude_success("late"), 0, 30);
        let cancel = CancelToken::default();
        let trigger = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(500));
            trigger.cancel();
        });
        let started = Instant::now();
        let result = generate(
            CliKind::Claude,
            &fake.program,
            "test-model",
            PROMPT,
            Duration::from_secs(60),
            &cancel,
        );
        assert_eq!(result, Err(CliFailure::Cancelled));
        assert!(started.elapsed() < Duration::from_secs(15));
    }

    #[test]
    fn missing_program_is_not_installed() {
        let missing = std::env::temp_dir().join("bob-no-such-cli-binary.exe");
        assert_eq!(
            generate(
                CliKind::Claude,
                &missing,
                "m",
                PROMPT,
                Duration::from_secs(5),
                &CancelToken::default()
            ),
            Err(CliFailure::NotInstalled)
        );
        let status = detect(CliKind::Codex, None);
        assert!(!status.installed);
    }

    #[test]
    fn auth_detection_reports_method_without_identity() {
        let report = parse_claude_auth(
            r#"{"loggedIn":true,"authMethod":"claude.ai","email":"a@b.c","orgId":"o","subscriptionType":"max"}"#,
        );
        assert_eq!(
            report,
            (
                AuthState::Ready,
                Some("account_session".into()),
                Some("max".into())
            )
        );
        assert_eq!(
            parse_claude_auth(r#"{"loggedIn":false}"#),
            (AuthState::Missing, None, None)
        );
        assert_eq!(parse_claude_auth("garbage").0, AuthState::Unknown);
        let output = |stdout: &str, success| ProcessOutput {
            stdout: stdout.into(),
            stderr: String::new(),
            success,
        };
        assert_eq!(
            parse_codex_auth(&output("Logged in using ChatGPT", true)).1,
            Some("account_session".into())
        );
        assert_eq!(
            parse_codex_auth(&output("Logged in using an API key - sk-proj-***abc", true)).1,
            Some("api_key".into())
        );
        assert_eq!(
            parse_codex_auth(&output("Not logged in", false)).0,
            AuthState::Missing
        );
        assert_eq!(
            parse_version("2.1.302 (Claude Code)"),
            Some("2.1.302".into())
        );
        assert_eq!(parse_version("codex-cli 0.98.0"), Some("0.98.0".into()));
    }

    #[test]
    fn detection_against_fake_cli_never_sends_a_prompt() {
        let fake = install(r#"{"loggedIn":true,"authMethod":"claude.ai"}"#, 0, 0);
        let status = detect(CliKind::Claude, Some(&fake.program));
        assert!(status.installed);
        assert_eq!(status.auth_state, AuthState::Ready);
        assert!(fake.stdin().trim().is_empty());
    }
}
