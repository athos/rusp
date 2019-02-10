use std::rc::Rc;
use std::result;
use crate::object::{self, Object};

type Frame = Vec<Rc<Object>>;

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

    fn frame(&self) -> Result<&Frame> {
        match self {
            Env::Frame(ref f, _) => Ok(f),
            _ => Err(object::Error)
        }
    }

    pub fn locate(&self, (i, j): Location) -> Result<Rc<Object>> {
        let mut env = self;
        let mut frame = self.frame()?;
        for _ in 0..i {
            match env {
                Env::Frame(f, next) => {
                    frame = f;
                    env = next;
                }
                _ => return Err(object::Error)
            }
        }
        Ok(frame.get(j).ok_or(object::Error)?.clone())
    }
}

pub fn push(env: Rc<Env>, frame: Frame) -> Env {
    Env::Frame(frame.clone(), env)
}
