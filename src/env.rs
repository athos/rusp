use std::rc::Rc;
use std::result;
use crate::error::{Error, error};
use crate::object::Object;

type Frame = Vec<Rc<Object>>;

#[derive(Debug, PartialEq)]
pub enum Env {
    Empty,
    Frame(Frame, Rc<Env>)
}

pub type Location = (usize, usize);

pub type Result<T> = result::Result<T, Error>;

impl Env {
    pub fn new() -> Self {
        Env::Empty
    }

    pub fn pop(self) -> Result<Rc<Env>> {
        match self {
            Env::Frame(_, next) => Ok(next),
            _ => Err(error("Env underflow"))
        }
    }

    pub fn locate(&self, (i, j): Location) -> Result<Rc<Object>> {
        let mut env = self;
        let mut frame: Option<&Frame> = None;
        for _ in 0..i+1 {
            match env {
                &Env::Frame(ref f, ref next) => {
                    frame = Some(f);
                    env = next;
                }
                _ => return Err(error("Illegal access to lexical environment"))
            }
        }
        if let Some(frame) = frame {
            Ok(frame.get(j).ok_or(error("Illegal access to lexical environment"))?.clone())
        } else {
            Err(error("Illegal access to lexical environment"))
        }
    }
}

pub fn push(env: Rc<Env>, frame: Frame) -> Env {
    Env::Frame(frame, env)
}

#[test]
fn locate_test() {
    let env = push(
        Rc::new(push(
            Rc::new(Env::new()),
            vec![
                Rc::new(Object::Number(1)),
                Rc::new(Object::Number(2))
            ]
        )),
        vec![Rc::new(Object::Number(0))]
    );
    assert_eq!(env.locate((1, 1)).expect("env should be high enough"),
               Rc::new(Object::Number(2)));
}
