use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn planning_request_round_trips_over_stdio() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bob-capability-host"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start capability host");

    let request = r#"{"protocolVersion":1,"requestId":"test-1","method":"planRemainingWork","params":{"activeId":"current","items":[{"id":"urgent","kind":"task","priority":"high","due":"Today","status":"planned"},{"id":"current","kind":"task","priority":"low","due":null,"status":"planned"},{"id":"doing","kind":"task","priority":"normal","due":null,"status":"doing"}]}}"#;

    {
        let stdin = child.stdin.as_mut().expect("host stdin");
        writeln!(stdin, "{request}").expect("write request");
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("read host response");
    assert!(output.status.success());

    let response: Value = serde_json::from_slice(&output.stdout).expect("valid JSON response");
    assert_eq!(response["ok"], true);
    assert_eq!(response["requestId"], "test-1");
    assert_eq!(response["result"]["nextId"], "current");
    assert_eq!(
        response["result"]["focusIds"],
        serde_json::json!(["current", "doing", "urgent"])
    );
}
