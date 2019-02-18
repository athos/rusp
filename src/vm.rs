use std::mem;
use std::rc::Rc;
use std::result;
use crate::env;
use crate::error::{Error, error};
use crate::insns::{Code, Insn};
use crate::insns::Insn::*;
use crate::object::{self, Object};

pub type Result<T> = result::Result<T, Error>;

type Pc = usize;
type Stack = Vec<Rc<Object>>;
type Env = Rc<env::Env>;
enum DumpEntry {
    Sel(Code, Pc),
    Ap(Stack, Env, Code, Pc)
}
type Dump = Vec<DumpEntry>;

pub struct Vm {
    stack: Stack,
    env: Env,
    code: Code,
    dump: Dump,
    pc: Pc
}

impl Vm {
    pub fn new(code: Code) -> Self {
        Vm { stack: vec![],
             env: Rc::new(env::Env::new()),
             code: code,
             dump: vec![],
             pc: 0 }
    }

    fn fetch_insn(&self) -> Option<Insn> {
        if self.pc < self.code.len() {
            Some(self.code[self.pc].clone())
        } else {
            None
        }
    }

    fn push(&mut self, v: Rc<Object>) {
        self.stack.push(v);
    }

    fn pop(&mut self) -> Result<Rc<Object>> {
        self.stack.pop().ok_or(error("Stack underflow"))
    }

    fn dump_pop(&mut self) -> Result<DumpEntry> {
        self.dump.pop().ok_or(error("Dump stack underflow"))
    }

    fn binary_op(&mut self, op: impl FnOnce(i32, i32) -> Rc<Object>) -> Result<()> {
        let y = self.pop()?.to_number()?;
        let x = self.pop()?.to_number()?;
        self.push(op(x, y));
        Ok(())
    }

    fn arith_op(&mut self, op: impl FnOnce(i32, i32) -> i32) -> Result<()> {
        self.binary_op(|x, y| Rc::new(Object::Number(op(x, y))))
    }

    fn logical_op(&mut self, op: impl FnOnce(i32, i32) -> bool) -> Result<()> {
        self.binary_op(|x, y| Rc::new(object::from_bool(op(x, y))))
    }

    pub fn run(&mut self) -> Result<Rc<Object>> {
        while let Some(insn) = self.fetch_insn() {
            match insn {
                Inil => self.push(Rc::new(Object::Nil)),
                Ildc(obj) => self.push(obj.clone()),
                Ild(loc) => {
                    let obj = self.env.locate(loc)?;
                    self.push(obj);
                }
                Iatom => {
                    let obj = self.pop()?;
                    self.push(Rc::new(object::from_bool(obj.is_atom())));
                }
                Inull => {
                    let obj = self.pop()?;
                    self.push(Rc::new(object::from_bool(obj.is_null())));
                }
                Icons => {
                    let x = self.pop()?;
                    let y = self.pop()?;
                    self.push(Rc::new(object::cons(x, y)));
                }
                Icar => {
                    let obj = self.pop()?;
                    self.push(obj.car()?);
                }
                Icdr => {
                    let obj = self.pop()?;
                    self.push(obj.cdr()?);
                }
                Iadd => self.arith_op(std::ops::Add::add)?,
                Isub => self.arith_op(std::ops::Sub::sub)?,
                Imul => self.arith_op(std::ops::Mul::mul)?,
                Idiv => self.arith_op(std::ops::Div::div)?,
                Ieq  => self.logical_op(|x, y| x == y)?,
                Igt  => self.logical_op(|x, y| x > y)?,
                Ilt  => self.logical_op(|x, y| x < y)?,
                Igte => self.logical_op(|x, y| x >= y)?,
                Ilte => self.logical_op(|x, y| x <= y)?,
                Isel(ct, cf) => {
                    self.run_sel(ct, cf)?;
                    continue;
                }
                Ijoin => self.run_join()?,
                Ildf(code) => {
                    let env = self.env.clone();
                    let obj = Object::Func(code.clone(), env);
                    self.push(Rc::new(obj));
                }
                Iap => {
                    self.run_ap()?;
                    continue;
                }
                Irtn => self.run_rtn()?
            }
            self.pc += 1;
        }
        Ok(self.pop().unwrap_or(Rc::new(Object::Nil)))
    }

    fn run_sel(&mut self, ct: Code, cf: Code) -> Result<()> {
        let mut c;
        if self.pop()?.to_bool() {
            c = ct;
        } else {
            c = cf;
        }
        mem::swap(&mut self.code, &mut c);
        self.dump.push(DumpEntry::Sel(c, self.pc));
        self.pc = 0;

        Ok(())
    }

    fn run_join(&mut self) -> Result<()> {
        match self.dump_pop()? {
            DumpEntry::Sel(code, pc) => {
                self.code = code;
                self.pc = pc;
                Ok(())
            }
            _ => Err(error("Run into incoherent dump entry (ap)"))
        }
    }

    fn run_ap(&mut self) -> Result<()> {
        match *self.pop()? {
            Object::Func(ref code, ref env) => {
                let args = self.pop()?;
                let frame = object::list_to_vec(args.as_ref())?;
                let stack = mem::replace(&mut self.stack, vec![]);
                let new_env = env::push(env.clone(), frame);
                let env = mem::replace(&mut self.env, Rc::new(new_env));
                let code = mem::replace(&mut self.code, code.clone());
                let entry = DumpEntry::Ap(stack, env, code, self.pc);
                self.dump.push(entry);
                self.pc = 0;
                Ok(())
            }
            _ => Err(error("Can't apply object other than function"))
        }
    }

    fn run_rtn(&mut self) -> Result<()> {
        let v = self.pop()?;
        match self.dump_pop()? {
            DumpEntry::Ap(mut stack, env, code, pc) => {
                stack.push(v);
                self.stack = stack;
                self.env = env;
                self.code = code;
                self.pc = pc;
                Ok(())
            }
            _ => Err(error("Run into incoherent dump entry (sel)"))
        }
    }
}

#[test]
fn vm_test() {
    let code = Rc::new(vec![
        Inil,
        Ildf(Rc::new(vec![
            Ildf(Rc::new(vec![
                Inil,
                Inil,
                Ild((0, 0)),
                Icons,
                Ild((1, 0)),
                Iap,
                Icons,
                Ild((1, 0)),
                Iap,
                Irtn
            ])),
            Irtn
        ])),
        Icons,
        Ildf(Rc::new(vec![
            Inil,
            Ildc(Rc::new(Object::Number(3))),
            Icons,
            Inil,
            Ildf(Rc::new(vec![
                Ild((0, 0)),
                Ildc(Rc::new(Object::Number(2))),
                Imul,
                Irtn
            ])),
            Icons,
            Ild((0, 0)),
            Iap,
            Iap,
            Irtn
        ])),
        Iap
    ]);
    let mut vm = Vm::new(code);
    let v = vm.run().expect("VM never fails");
    assert_eq!(v, Rc::new(Object::Number(12)));
}
