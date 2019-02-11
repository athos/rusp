use std::rc::Rc;
use std::result;
use crate::object::{self, Object};

type Frame = Vec<Rc<Object>>;

#[derive(Debug, PartialEq)]
pub enum Env {
    Empty,
    Frame(Frame, Rc<Env>)
}

pub type Location = (usize, usize);

pub type Result<T> = result::Result<T, object::Error>;

impl Env {
    pub fn new() -> Self {
        Env::Empty
    }

    pub fn pop(self) -> Result<Rc<Env>> {
        match self {
            Env::Frame(_, next) => Ok(next),
            _ => Err(object::Error)
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
                _ => return Err(object::Error)
            }
        }
        if let Some(frame) = frame {
            Ok(frame.get(j).ok_or(object::Error)?.clone())
        } else {
            Err(object::Error)
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
