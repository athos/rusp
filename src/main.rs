extern crate rusp;

use std::io::{self, BufRead, Write};
use rusp::compiler as comp;
use rusp::reader;
use rusp::vm::Vm;

fn prompt() -> io::Result<()> {
    print!("> ");
    io::stdout().flush()?;

    Ok(())
}

fn main() -> io::Result<()> {
    prompt()?;
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Some(expr) = reader::read_string(&line?) {
            let code = comp::compile(&expr).unwrap();
            let mut vm = Vm::new(code);
            let v = vm.run().unwrap();
            println!("{}", *v);
        }
        prompt()?;
    }
    Ok(())
}
