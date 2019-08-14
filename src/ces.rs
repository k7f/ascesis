use std::{ops::Deref, error::Error};
use aces::{Content, PartialContent, ContextHandle, NodeID};
use crate::{VisBlock, CapacityBlock, MultiplierBlock, InhibitorBlock, Rex, rex::RexKind, AscesisError};

#[derive(Default, Debug)]
pub struct CesFile {
    blocks:  Vec<CesFileBlock>,
    root:    Option<usize>,
    script:  Option<String>,
    content: Option<PartialContent>,
}

impl CesFile {
    pub fn from_script(script: String) -> Result<Self, Box<dyn Error>> {
        let mut result: Self = script.parse()?;

        result.script = Some(script);

        Ok(result)
    }

    pub fn set_root_name<S: AsRef<str>>(&mut self, root_name: S) -> Result<(), Box<dyn Error>> {
        let root_name = root_name.as_ref();

        self.root = None;

        for (ndx, block) in self.blocks.iter().enumerate() {
            if let CesFileBlock::Imm(imm) = block {
                if imm.name.0.as_str() == root_name {
                    if self.root.is_none() {
                        self.root = Some(ndx);
                    } else {
                        return Err(Box::new(AscesisError::RootRedefined(root_name.to_owned())))
                    }
                }
            }
        }

        if self.root.is_some() {
            Ok(())
        } else {
            Err(Box::new(AscesisError::RootMissing(root_name.to_owned())))
        }
    }

    fn get_root(&self) -> Result<&ImmediateDef, AscesisError> {
        if let Some(ndx) = self.root {
            if let Some(block) = self.blocks.get(ndx) {
                if let CesFileBlock::Imm(ref root) = block {
                    Ok(root)
                } else {
                    Err(AscesisError::RootBlockMismatch)
                }
            } else {
                Err(AscesisError::RootBlockMissing)
            }
        } else {
            Err(AscesisError::RootUnset)
        }
    }

    pub fn compile(&mut self, ctx: ContextHandle) -> Result<(), AscesisError> {
        let root = self.get_root()?;
        let rex = root.rex.fit_clone();
        let mut content = PartialContent::new();

        for kind in rex.kinds.iter() {
            match kind {
                RexKind::Thin(tar) => {
                    let cause = tar.get_flattened_cause().into_content(ctx.clone());
                    let effect = tar.get_flattened_effect().into_content(ctx.clone());

                    for node in tar.get_nodes() {
                        let mut ctx = ctx.lock().unwrap();
                        let id = ctx.share_node_name(node);
                        println!("{:?} {:?} C {:?} E {:?}", node, id, cause, effect);

                        if !cause.is_empty() {
                            content.add_to_causes(id, &cause);
                        }

                        if !effect.is_empty() {
                            content.add_to_effects(id, &effect);
                        }
                    }
                }
                _ => {}
            }
        }

        self.content = Some(content);

        Ok(())
    }
}

impl From<Vec<CesFileBlock>> for CesFile {
    fn from(blocks: Vec<CesFileBlock>) -> Self {
        CesFile { blocks, ..Default::default() }
    }
}

impl Content for CesFile {
    fn get_script(&self) -> Option<&str> {
        self.script.as_ref().map(|s| s.as_str())
    }

    fn get_name(&self) -> Option<&str> {
        let root = self.get_root().unwrap();

        Some(root.name.as_str())
    }

    fn get_carrier_ids(&mut self) -> Vec<NodeID> {
        if let Some(ref mut content) = self.content {
            content.get_carrier_ids()
        } else {
            panic!()
        }
    }

    fn get_causes_by_id(&self, id: NodeID) -> Option<&Vec<Vec<NodeID>>> {
        if let Some(ref content) = self.content {
            content.get_causes_by_id(id)
        } else {
            panic!()
        }
    }

    fn get_effects_by_id(&self, id: NodeID) -> Option<&Vec<Vec<NodeID>>> {
        if let Some(ref content) = self.content {
            content.get_effects_by_id(id)
        } else {
            panic!()
        }
    }
}

#[derive(Debug)]
pub enum CesFileBlock {
    Imm(ImmediateDef),
    Vis(VisBlock),
    Cap(CapacityBlock),
    Mul(MultiplierBlock),
    Inh(InhibitorBlock),
}

impl From<ImmediateDef> for CesFileBlock {
    fn from(imm: ImmediateDef) -> Self {
        CesFileBlock::Imm(imm)
    }
}

impl From<VisBlock> for CesFileBlock {
    fn from(vis: VisBlock) -> Self {
        CesFileBlock::Vis(vis)
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct CesName(String);

impl Deref for CesName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for CesName {
    fn from(name: String) -> Self {
        CesName(name)
    }
}

pub trait ToCesName {
    fn to_ces_name(&self) -> CesName;
}

impl<S: AsRef<str>> ToCesName for S {
    fn to_ces_name(&self) -> CesName {
        self.as_ref().to_string().into()
    }
}

#[derive(Clone, Debug)]
pub struct ImmediateDef {
    name: CesName,
    rex:  Rex,
}

impl ImmediateDef {
    pub fn new(name: CesName, rex: Rex) -> Self {
        ImmediateDef { name, rex }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CesInstance {
    pub(crate) name: CesName,
    pub(crate) args: Vec<String>,
}

impl CesInstance {
    pub(crate) fn new(name: CesName) -> Self {
        CesInstance { name, args: Vec::new() }
    }

    pub(crate) fn with_args(mut self, mut args: Vec<String>) -> Self {
        self.args.append(&mut args);
        self
    }
}
