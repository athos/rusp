use std::collections::HashMap;
use crate::object::Object;

pub type CodeAddr = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Insn {
    Inil,
    Ildc(Object),
    Ild(u32, u32),
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
    Isel(CodeAddr, CodeAddr),
    Ijoin
}

pub type Code = Vec<Insn>;
pub type Program = HashMap<CodeAddr, Code>;
