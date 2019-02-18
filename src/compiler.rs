use std::collections::HashMap;
use std::rc::Rc;
use std::result;
use crate::env::Location;
use crate::error::{Error, error};
use crate::insns::{Code, Insn};
use crate::insns::Insn::*;
use crate::object::{self, Object};
use crate::object::Object::*;

pub type Result<T> = result::Result<T, Error>;
type CEnv = HashMap<String, Location>;

#[derive(Debug, Clone)]
struct Compiler {
    insns: Vec<Insn>,
    cenv: CEnv,
    level: usize
}

impl Compiler {
    fn new() -> Self {
        Compiler {
            insns: Vec::new(),
            cenv: HashMap::new(),
            level: 0
        }
    }

    fn renew(&self) -> Self {
        Compiler {
            insns: Vec::new(),
            cenv: self.cenv.clone(),
            level: self.level
        }
    }

    fn compile(&mut self, expr: &Object) -> Result<()> {
        match expr {
            Nil => self.insns.push(Inil),
            T | Number(_) => {
                let obj = expr.clone();
                self.insns.push(Ildc(Rc::new(obj)));
            }
            Symbol(ref name) => {
                let (i, j) = self.cenv.get(name).ok_or_else(|| {
                    let msg = format!("unknown variable: {}", *name);
                    error(&msg)
                })?;
                self.insns.push(Ild((self.level - i, *j)));
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
                    _ => self.compile_application(car, cdr)?
                }
            }
            Nil | T | Number(_) => {
                let msg = format!("{} is not applicable", *car);
                return Err(error(&msg));
            }
            _ => self.compile_application(car, cdr)?
        }
        Ok(())
    }

    fn take_args(&self, n: usize, args: &Object) -> Result<Vec<Rc<Object>>> {
        let args = object::list_to_vec(args).or_else(|_| {
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
        let mut c1 = self.renew();
        c1.compile(args[1].as_ref())?;
        let mut c2 = self.renew();
        c2.compile(args[2].as_ref())?;
        self.insns.push(Isel(Rc::new(c1.insns), Rc::new(c2.insns)));
        Ok(())
    }

    fn compile_lambda(&mut self, args: &Object) -> Result<()> {
        let args = self.take_args(2, args)?;
        let mut c = self.renew();
        c.level += 1;
        for (i, arg) in object::list_to_vec(args[0].as_ref())?.iter().enumerate() {
            match arg.as_ref() {
                Symbol(ref name) => {
                    c.cenv.insert(name.to_owned(), (c.level, i));
                }
                _ => return Err(error("fn argument must be symbol"))
            }
        }
        c.compile(args[1].as_ref())?;
        c.insns.push(Irtn);
        self.insns.push(Ildf(Rc::new(c.insns)));
        Ok(())
    }

    fn compile_application(&mut self, func: &Object, args: &Object) -> Result<()> {
        let args = object::list_to_vec(args)?;
        self.insns.push(Inil);
        for arg in args.iter().rev() {
            self.compile(arg.as_ref())?;
            self.insns.push(Icons);
        }
        self.compile(func)?;
        self.insns.push(Iap);
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
