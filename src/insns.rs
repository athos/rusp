use crate::object::Object;

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
    Ilte
}

pub type Code = Vec<Insn>;
