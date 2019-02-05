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
        let obj = reader::read_string(&line?).unwrap();
        print!("{:?}\n", obj);
        prompt()?;
    }

    Ok(())
}
