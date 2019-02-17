use std::rc::Rc;
use std::result;
use crate::error::{Error, error};
use crate::insns::{Code, Insn};
use crate::insns::Insn::*;
use crate::object::{self, Object};
use crate::object::Object::*;

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
                    "+"  => self.compile_op(2, cdr, Iadd)?,
                    "-"  => self.compile_op(2, cdr, Isub)?,
                    "*"  => self.compile_op(2, cdr, Imul)?,
                    "/"  => self.compile_op(2, cdr, Idiv)?,
                    "="  => self.compile_op(2, cdr, Ieq )?,
                    "<"  => self.compile_op(2, cdr, Ilt )?,
                    ">"  => self.compile_op(2, cdr, Igt )?,
                    "<=" => self.compile_op(2, cdr, Ilte)?,
                    ">=" => self.compile_op(2, cdr, Igte)?,
                    "cons" => self.compile_op(2, cdr, Icons)?,
                    "car"  => self.compile_op(1, cdr, Icar)?,
                    "cdr"  => self.compile_op(1, cdr, Icdr)?,
                    "null" => self.compile_op(1, cdr, Inull)?,
                    "atom" => self.compile_op(1, cdr, Iatom)?,
                    "if" => self.compile_if(cdr)?,
                    "lambda" => self.compile_lambda(cdr)?,
                    _ => unimplemented!()
                }
            }
            _ => unimplemented!()
        }
        Ok(())
    }

    fn take_args(&self, n: usize, args: &Object) -> Result<Vec<Rc<Object>>> {
        let args = object::list_to_vec(Rc::new(args.clone())).or_else(|_| {
            Err(error("arglist must be proper list"))
        })?;
        let nargs = args.len();
        if nargs < n {
            return Err(error("too less arguments"))
        } else if nargs > n {
            return Err(error("too many arguments"))
        }
        Ok(args)
    }

    fn compile_op(&mut self, n: usize, args: &Object, insn: Insn) -> Result<()> {
        let args = self.take_args(n, args)?;
        for arg in args {
            self.compile(arg.as_ref())?;
        }
        self.insns.push(insn);
        Ok(())
    }

    fn compile_if(&mut self, args: &Object) -> Result<()> {
        let args = self.take_args(3, args)?;
        self.compile(args[0].as_ref())?;
        let then = compile(args[1].as_ref())?;
        let otherwise = compile(args[2].as_ref())?;
        self.insns.push(Isel(then, otherwise));
        Ok(())
    }

    fn compile_lambda(&mut self, args: &Object) -> Result<()> {
        let args = self.take_args(2, args)?;
        let _fn_args = object::list_to_vec(args[0].clone());
        let fn_body = compile(args[1].as_ref())?;
        self.insns.push(Ildf(fn_body));
        Ok(())
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
