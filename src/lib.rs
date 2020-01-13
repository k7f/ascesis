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

mod error;
mod bnf;
pub mod grammar;
pub mod sentence;
mod axiom;
mod ces;
mod context;
mod content;
mod rex;
mod polynomial;
mod node;
mod lex;

pub use aces::*;

pub use error::{AscesisError, ParsingError, ParsingResult};
pub use axiom::Axiom;
pub use ces::{CesFile, CesFileBlock, CesName, ToCesName, ImmediateDef, CesInstance};
pub use context::{
    PropBlock, PropSelector, PropValue, CapacityBlock, MultiplicityBlock, InhibitorBlock,
};
pub use content::AscesisFormat;
pub use rex::{Rex, ThinArrowRule, FatArrowRule};
pub use polynomial::Polynomial;
pub use node::{Node, ToNode, NodeList};
pub use lex::{Literal, BinOp};
