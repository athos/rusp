use crate::insns::Code;
use crate::insns::Insn::{self, *};
use crate::object::{self, Object};

type Stack = Vec<Object>;
type Env = Vec<Object>;
type DumpEntry = (Stack, Env, Code);
type Dump = Vec<DumpEntry>;

pub struct Vm {
    stack: Stack,
    env: Env,
    code: Code,
    dump: Dump,
    pc: usize
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
                _ => break
            }
            self.pc += 1;
        }
    }
}

#[test]
fn vm_test() {
    let code = vec![
        Ildc(Object::Number(1)),
        Ildc(Object::Number(2)),
        Iadd,
        Ildc(Object::Number(3)),
        Imul
    ];
    let mut vm = Vm::new(code);
    vm.run();
    assert_eq!(vm.stack, vec![Object::Number(9)]);
}
