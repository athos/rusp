use std::rc::Rc;
use std::result;
use crate::error::{Error, error};
use crate::insns::{Code, Insn};
use crate::insns::Insn::*;
use crate::object::Object::{self, *};

pub type Result<T> = result::Result<T, Error>;

struct Compiler {
    insns: Vec<Insn>
}

impl Compiler {
    fn new() -> Self {
        Compiler { insns: Vec::new() }
    }

    fn compile(&mut self, expr: &Object) -> Result<()> {
        match expr {
            Nil => self.insns.push(Inil),
            T | Number(_) => {
                let obj = expr.clone();
                self.insns.push(Ildc(Rc::new(obj)));
            }
            Cons(car, cdr) => self.compile_list(car, cdr)?,
            _ => unimplemented!()
        }
        Ok(())
    }

    fn compile_list(&mut self, car: &Object, cdr: &Object) -> Result<()> {
        match car {
            Symbol(ref name) => {
                match name.as_ref() {
                    "+"  => self.compile_binary_op(cdr, Iadd)?,
                    "-"  => self.compile_binary_op(cdr, Isub)?,
                    "*"  => self.compile_binary_op(cdr, Imul)?,
                    "/"  => self.compile_binary_op(cdr, Idiv)?,
                    "="  => self.compile_binary_op(cdr, Ieq )?,
                    "<"  => self.compile_binary_op(cdr, Ilt )?,
                    ">"  => self.compile_binary_op(cdr, Igt )?,
                    "<=" => self.compile_binary_op(cdr, Ilte)?,
                    ">=" => self.compile_binary_op(cdr, Igte)?,
                    _ => unimplemented!()
                }
            }
            _ => unimplemented!()
        }
        Ok(())
    }

    fn compile_binary_op(&mut self, args: &Object, insn: Insn) -> Result<()> {
        match args {
            Cons(x, cdr) => {
                match cdr.as_ref() {
                    Cons(y, cddr) => {
                        match cddr.as_ref() {
                            Nil => {
                                self.compile(x)?;
                                self.compile(y)?;
                                self.insns.push(insn);
                                Ok(())
                            }
                            _ => Err(error(""))
                        }
                    }
                    _ => Err(error(""))
                }
            }
            _ => Err(error(""))
        }
    }
}

pub fn compile(expr: &Object) -> Result<Code> {
    let mut compiler = Compiler::new();
    compiler.compile(expr)?;
    Ok(Rc::new(compiler.insns))
}

#[test]
fn compile_test() {
    // (+ (* 3 3) (* 4 4))
    let code = compile(&Cons(
        Rc::new(Symbol("+".to_owned())),
        Rc::new(Cons(
            Rc::new(Cons(
                Rc::new(Symbol("*".to_owned())),
                Rc::new(Cons(
                    Rc::new(Number(3)),
                    Rc::new(Cons(
                        Rc::new(Number(3)),
                        Rc::new(Nil)
                    ))
                ))
            )),
            Rc::new(Cons(
                Rc::new(Cons(
                    Rc::new(Symbol("*".to_owned())),
                    Rc::new(Cons(
                        Rc::new(Number(4)),
                        Rc::new(Cons(
                            Rc::new(Number(4)),
                            Rc::new(Nil)
                        ))
                    ))
                )),
                Rc::new(Nil)
            ))
        ))
    )).expect("compile fails");
    let expected = Rc::new(vec![
        Ildc(Rc::new(Number(3))),
        Ildc(Rc::new(Number(3))),
        Imul,
        Ildc(Rc::new(Number(4))),
        Ildc(Rc::new(Number(4))),
        Imul,
        Iadd
    ]);
    assert_eq!(code, expected);
}
