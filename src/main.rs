mod rusp;

use std::io::{self, BufRead, Write};
use rusp::reader;

fn prompt() -> io::Result<()> {
    print!("> ");
    io::stdout().flush()?;

    Ok(())
}

fn main() -> io::Result<()> {
    prompt()?;
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Some(obj) = reader::read_string(&line?) {
            print!("{:?}\n", obj);
        }
        prompt()?;
    }
    Ok(())
}
