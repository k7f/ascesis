#![feature(slice_partition_dedup)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(
    #[allow(clippy::all)]
    pub ascesis_parser
);

lalrpop_mod!(
    #[allow(clippy::all)]
    pub bnf_parser
);

pub mod error;
pub mod bnf;
pub mod grammar;
pub mod sentence;
pub mod axiom;
pub mod ces;
pub mod context;
pub mod rex;
pub mod polynomial;
pub mod node;
pub mod lex;

pub use crate::error::{AscesisError, ParsingError, ParsingResult};
pub use crate::axiom::Axiom;
pub use crate::ces::{CesFile, CesFileBlock, CesName, ToCesName, ImmediateDef, CesInstance};
pub use crate::context::{
    PropBlock, PropSelector, PropValue, CapacityBlock, MultiplicityBlock, InhibitorBlock,
};
pub use crate::rex::{Rex, ThinArrowRule, FatArrowRule};
pub use crate::polynomial::Polynomial;
pub use crate::node::{Node, ToNode, NodeList};
pub use crate::lex::{Literal, BinOp};
