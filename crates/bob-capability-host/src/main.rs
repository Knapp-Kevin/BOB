use bob_capability_host::handle_line;
use std::io::{self, BufRead, Write};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::BufWriter::new(io::stdout().lock());

    for line in stdin.lock().lines() {
        let line = line?;
        let response = handle_line(&line);
        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }

    Ok(())
}
