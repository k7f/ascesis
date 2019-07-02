#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub cesar);

#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    String(String),
    Size(u64),
}
