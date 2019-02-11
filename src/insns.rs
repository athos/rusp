use crate::object::Object;
use crate::env::Location;

use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Insn {
    Inil,
    Ildc(Rc<Object>),
    Ild(Location),
    Iatom,
    Inull,
    Icar,
    Icdr,
    Icons,
    Iadd,
    Isub,
    Imul,
    Idiv,
    Ieq,
    Igt,
    Ilt,
    Igte,
    Ilte,
    Isel(Code, Code),
    Ijoin,
    Ildf(Code),
    Iap,
    Irtn
}

pub type Code = Rc<Vec<Insn>>;
