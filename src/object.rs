use std::fmt;
use std::rc::Rc;
use std::result;
use crate::env::Env;
use crate::error::{Error, error};
use crate::insns::Code;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Nil,
    T,
    Number(i32),
    Symbol(String),
    Cons(Rc<Object>, Rc<Object>),
    Func(Code, Rc<Env>)
}

pub type Result<T> = result::Result<T, Error>;

fn write_list(f: &mut fmt::Formatter, obj: &Object) -> fmt::Result {
    match *obj {
        Object::Cons(ref car, ref cdr) => {
            write!(f, "{}", car)?;
            match **cdr {
                Object::Nil => Ok(()),
                _ => {
                    write!(f, " ")?;
                    write_list(f, cdr)
                }
            }
        }
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
            Object::Cons(..) => {
                write!(f, "(")?;
                write_list(f, self)?;
                write!(f, ")")
            }
            Object::Func(..) => write!(f, "#<func>")
        }
    }
}

impl Object {
    pub fn is_atom(&self) -> bool {
        match self {
            Object::Cons(..) => false,
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
            _ => Err(error("Can't be converted to number"))
        }
    }

    pub fn car(&self) -> Result<Rc<Object>> {
        match self {
            &Object::Nil => Ok(Rc::new(Object::Nil)),
            &Object::Cons(ref car, _) => Ok(car.clone()),
            _ => Err(error("Can't be converted to cons"))
        }
    }

    pub fn cdr(&self) -> Result<Rc<Object>> {
        match self {
            &Object::Nil => Ok(Rc::new(Object::Nil)),
            &Object::Cons(_, ref cdr) => Ok(cdr.clone()),
            _ => Err(error("Can't be converted to cons"))
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

pub fn cons(car: Rc<Object>, cdr: Rc<Object>) -> Object {
    Object::Cons(car, cdr)
}

pub fn list_to_vec(mut obj: &Object) -> Result<Vec<Rc<Object>>> {
    let mut ret = Vec::new();
    loop {
        match obj {
            &Object::Nil => break,
            &Object::Cons(ref car, ref cdr) => {
                ret.push(car.clone());
                obj = cdr.as_ref();
            }
            _ => return Err(error("Can't be converted to cons"))
        }
    }
    Ok(ret)
}

#[test]
fn list_to_vec_test() {
    assert_eq!(list_to_vec(Rc::new(Object::Nil)).expect("must not happen"), vec![]);
    assert_eq!(list_to_vec(Rc::new(Object::Cons(
        Rc::new(Object::Number(1)),
        Rc::new(Object::Cons(
            Rc::new(Object::Number(2)),
            Rc::new(Object::Cons(
                Rc::new(Object::Number(3)),
                Rc::new(Object::Nil)
            ))
        ))
    ))).expect("must not happpen"), vec![
        Rc::new(Object::Number(1)),
        Rc::new(Object::Number(2)),
        Rc::new(Object::Number(3))
    ]);
}
