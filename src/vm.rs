use std::collections::HashMap;
use crate::insns::{Code, CodeAddr, Program};
use crate::insns::Insn::{self, *};
use crate::object::{self, Object};

type Pc = usize;
type Stack = Vec<Object>;
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

    fn push(&mut self, v: Object) {
        self.stack.push(v);
    }

    fn pop(&mut self) -> Option<Object> {
        self.stack.pop()
    }

    fn arith_op(&mut self, op: impl Fn(i32, i32) -> i32) {
        let x = self.pop().unwrap();
        let y = self.pop().unwrap();
        match x {
            Object::Number(m) => {
                match y {
                    Object::Number(n) => {
                        self.push(Object::Number(op(m, n)));
                    },
                    _ => ()
                }
            },
            _ => ()
        }
    }

    pub fn run(&mut self) {
        while let Some(insn) = self.fetch_insn() {
            match insn {
                Inil => self.push(Object::Nil),
                Ildc(obj) => self.push(obj),
                Iatom => {
                    let obj = self.pop().unwrap();
                    self.push(object::from_bool(obj.is_atom()));
                },
                Inull => {
                    let obj = self.pop().unwrap();
                    self.push(object::from_bool(obj.is_null()));
                },
                Icons => {
                    let x = self.pop().unwrap();
                    let y = self.pop().unwrap();
                    self.push(object::cons(x, y));
                },
                Iadd => self.arith_op(std::ops::Add::add),
                Isub => self.arith_op(std::ops::Sub::sub),
                Imul => self.arith_op(std::ops::Mul::mul),
                Idiv => self.arith_op(std::ops::Div::div),
                Isel(ct, cf) => {
                    let c;
                    if self.pop().unwrap().to_bool() {
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
                        _ => ()
                    }
                }
                _ => break
            }
            self.pc += 1;
        }
    }
}

#[test]
fn vm_test() {
    let mut program: Program = vec![
        (0, vec![
            Ildc(Object::Nil),
            Inull,
            Isel(1, 2),
            Ildc(Object::Number(3)),
            Iadd
        ]),
        (1, vec![
            Ildc(Object::Number(1)),
            Ijoin
        ]),
        (2, vec![
            Ildc(Object::Number(2)),
            Ijoin
        ])
    ].iter().cloned().collect();
    let mut vm = Vm::new(&program, 0);
    vm.run();
    assert_eq!(vm.stack, vec![Object::Number(4)]);
}
