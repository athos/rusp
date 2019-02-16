extern crate rusp;

use std::io::{self, BufRead, Write};
use rusp::compiler as comp;
use rusp::error::Error;
use rusp::object::Object;
use rusp::reader;
use rusp::vm::Vm;

fn prompt() -> io::Result<()> {
    print!("> ");
    io::stdout().flush()?;

    Ok(())
}

fn step(expr: &Object) -> Result<(), Error> {
    let code = comp::compile(&expr)?;
    let mut vm = Vm::new(code);
    let v = vm.run()?;
    println!("{}", *v);

    Ok(())
}

fn main() -> io::Result<()> {
    prompt()?;
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        if let Some(expr) = reader::read_string(&line?) {
            if let Err(err) = step(&expr) {
                println!("Error: {}", err);
            }
        }
        prompt()?;
    }
    Ok(())
}
