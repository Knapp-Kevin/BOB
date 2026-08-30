use bob_capability_host::{handle_line, MAX_MESSAGE_BYTES};
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut input = Vec::with_capacity(MAX_MESSAGE_BYTES + 2);
    stdin
        .lock()
        .take((MAX_MESSAGE_BYTES + 2) as u64)
        .read_to_end(&mut input)?;

    let response = if input.len() > MAX_MESSAGE_BYTES + 1 {
        handle_line(&"x".repeat(MAX_MESSAGE_BYTES + 1))
    } else {
        let request = String::from_utf8_lossy(&input);
        handle_line(request.trim_end_matches(|character| character == '\r' || character == '\n'))
    };

    let mut stdout = io::BufWriter::new(io::stdout().lock());
    writeln!(stdout, "{response}")?;
    stdout.flush()
}
