use crate::{CapacityBlock, MultiplierBlock, InhibitorBlock, Rex};

#[derive(Debug)]
pub enum CesFileBlock {
    Imm(ImmediateDef),
    Cap(CapacityBlock),
    Mul(MultiplierBlock),
    Inh(InhibitorBlock),
}

impl From<ImmediateDef> for CesFileBlock {
    fn from(imm: ImmediateDef) -> Self {
        CesFileBlock::Imm(imm)
    }
}

impl From<CapacityBlock> for CesFileBlock {
    fn from(cap: CapacityBlock) -> Self {
        CesFileBlock::Cap(cap)
    }
}

impl From<MultiplierBlock> for CesFileBlock {
    fn from(mul: MultiplierBlock) -> Self {
        CesFileBlock::Mul(mul)
    }
}

impl From<InhibitorBlock> for CesFileBlock {
    fn from(inh: InhibitorBlock) -> Self {
        CesFileBlock::Inh(inh)
    }
}

#[derive(Debug)]
pub struct ImmediateDef {
    name: String,
    rex:  Rex,
}

impl ImmediateDef {
    pub fn new(name: String, rex: Rex) -> Self {
        ImmediateDef { name, rex }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct CesInstance {
    pub(crate) name: String,
    pub(crate) args: Vec<String>,
}

impl CesInstance {
    pub(crate) fn new(name: String) -> Self {
        CesInstance { name, args: Vec::new() }
    }

    pub(crate) fn with_args(mut self, mut args: Vec<String>) -> Self {
        self.args.append(&mut args);
        self
    }
}
