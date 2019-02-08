use std::collections::HashMap;
use std::rc::Rc;
use std::result;
use crate::insns::{Code, CodeAddr, Program};
use crate::insns::Insn::{self, *};
use crate::object::{self, Object};

pub type Error = object::Error;
pub type Result<T> = result::Result<T, Error>;

type Pc = usize;
type Stack = Vec<Rc<Object>>;
type Env = Vec<Object>;
enum DumpEntry<'a> {
    Sel(&'a Code, Pc),
    Ap(Stack, Env, &'a Code, Pc)
}
type Dump<'a> = Vec<DumpEntry<'a>>;

pub struct Vm<'a> {
    stack: Stack,
    env: Env,
    code: &'a Code,
    dump: Dump<'a>,
    pc: Pc,
    program: &'a Program
}

impl<'a> Vm<'a> {
    pub fn new(program: &'a Program, entry_point: CodeAddr) -> Self {
        let code = program.get(&entry_point).unwrap();
        Vm { stack: vec![], env: vec![], code: code, dump: vec![], pc: 0, program }
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
                },
                Inull => {
                    let obj = self.pop()?;
                    self.push(Rc::new(object::from_bool(obj.is_null())));
                },
                Icons => {
                    let x = self.pop()?;
                    let y = self.pop()?;
                    self.push(Rc::new(object::cons(x, y)));
                },
                Icar => {
                    let obj = self.pop()?;
                    self.push(obj.car()?);
                },
                Icdr => {
                    let obj = self.pop()?;
                    self.push(obj.cdr()?);
                },
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
                    let c;
                    if self.pop()?.to_bool() {
                        c = ct;
                    } else {
                        c = cf;
                    }
                    self.dump.push(DumpEntry::Sel(self.code, self.pc+1));
                    self.code = self.program.get(&c).unwrap();
                    self.pc = 0;
                    continue;
                },
                Ijoin => {
                    match self.dump.pop().unwrap() {
                        DumpEntry::Sel(code, pc) => {
                            self.code = code;
                            self.pc = pc;
                            continue;
                        },
                        _ => unimplemented!()
                    }
                }
                _ => unimplemented!()
            }
            self.pc += 1;
        }
        Ok(())
    }
}

#[test]
fn vm_test() {
    let program: Program = vec![
        (0, vec![
            Ildc(Rc::new(Object::Nil)),
            Inull,
            Isel(1, 2),
            Ildc(Rc::new(Object::Number(3))),
            Iadd
        ]),
        (1, vec![
            Ildc(Rc::new(Object::Number(1))),
            Ijoin
        ]),
        (2, vec![
            Ildc(Rc::new(Object::Number(2))),
            Ijoin
        ])
    ].iter().cloned().collect();
    let mut vm = Vm::new(&program, 0);
    vm.run();
    assert_eq!(vm.stack, vec![Rc::new(Object::Number(4))]);
}
