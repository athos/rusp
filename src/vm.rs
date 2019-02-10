use std::mem;
use std::rc::Rc;
use std::result;
use crate::insns::{Code, Insn};
use crate::insns::Insn::*;
use crate::object::{self, Object};

pub type Error = object::Error;
pub type Result<T> = result::Result<T, Error>;

type Pc = usize;
type Stack = Vec<Rc<Object>>;
type Env = Vec<Object>;
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
        Vm { stack: vec![], env: vec![], code: code, dump: vec![], pc: 0 }
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
        self.stack.pop().ok_or(object::Error)
    }

    fn dump_pop(&mut self) -> Result<DumpEntry> {
        self.dump.pop().ok_or(object::Error)
    }

    fn binary_op(&mut self, op: impl FnOnce(i32, i32) -> Rc<Object>) -> Result<()> {
        let x = self.pop()?.to_number()?;
        let y = self.pop()?.to_number()?;
        self.push(op(x, y));
        Ok(())
    }

    fn arith_op(&mut self, op: impl FnOnce(i32, i32) -> i32) -> Result<()> {
        self.binary_op(|x, y| Rc::new(Object::Number(op(x, y))))
    }

    fn logical_op(&mut self, op: impl FnOnce(i32, i32) -> bool) -> Result<()> {
        self.binary_op(|x, y| Rc::new(object::from_bool(op(x, y))))
    }

    pub fn run(&mut self) -> Result<()> {
        while let Some(insn) = self.fetch_insn() {
            match insn {
                Inil => self.push(Rc::new(Object::Nil)),
                Ildc(obj) => self.push(obj.clone()),
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
                _ => unimplemented!()
            }
            self.pc += 1;
        }
        Ok(())
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
            _ => Err(object::Error)
        }
    }
}

#[test]
fn vm_test() {
    let code = Rc::new(vec![
        Ildc(Rc::new(Object::Nil)),
        Inull,
        Isel(Rc::new(vec![Ildc(Rc::new(Object::Number(1))), Ijoin]),
             Rc::new(vec![Ildc(Rc::new(Object::Number(2))), Ijoin])),
        Ildc(Rc::new(Object::Number(3))),
        Iadd
    ]);
    let mut vm = Vm::new(code);
    vm.run();
    assert_eq!(vm.stack, vec![Rc::new(Object::Number(4))]);
}
