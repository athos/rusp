use std::fmt;
use std::result;
use crate::insns::Code;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Nil,
    T,
    Number(i32),
    Symbol(String),
    Cons { car: Box<Object>, cdr: Box<Object> }
}

pub struct Error;
pub type Result<T> = result::Result<T, Error>;

fn write_list(f: &mut fmt::Formatter, obj: &Object) -> fmt::Result {
    match *obj {
        Object::Cons { ref car, ref cdr } => {
            write!(f, "{}", car);
            match **cdr {
                Object::Nil => Ok(()),
                _ => {
                    write!(f, " ");
                    write_list(f, cdr)
                }
            }
        },
        _ => write!(f, ". {}", obj)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Nil => write!(f, "nil"),
            Object::T => write!(f, "t"),
            Object::Number(ref num) => write!(f,"{}", num),
            Object::Symbol(ref sym) => write!(f, "{}", sym),
            Object::Cons { .. } => {
                write!(f, "(");
                write_list(f, self)?;
                write!(f, ")")
            }
        }
    }
}

impl Object {
    pub fn is_atom(&self) -> bool {
        match self {
            Object::Cons {..} | Object::Func {..} => false,
            _ => true
        }
    }

    pub fn is_null(&self) -> bool {
        *self == Object::Nil
    }

    pub fn to_bool(&self) -> bool {
        !self.is_null()
    }

    pub fn to_number(&self) -> Result<i32> {
        match *self {
            Object::Number(n) => Ok(n),
            _ => Err(Error)
        }
    }
}

pub fn from_bool(b: bool) -> Object {
    if b { Object::T } else { Object::Nil }
}

pub fn number(num: i32) -> Object {
    Object::Number(num)
}

pub fn symbol(name: &str) -> Object {
    Object::Symbol(name.to_string())
}

pub fn cons(car: Object, cdr: Object) -> Object {
    Object::Cons { car: Box::new(car), cdr: Box::new(cdr) }
}
