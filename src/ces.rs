use std::{ops::Deref, error::Error};
use aces::{Content, PartialContent, CompilableAsContent, ContextHandle, NodeID, sat};
use crate::{
    PropBlock, PropSelector, CapacityBlock, MultiplierBlock, InhibitorBlock, Rex, AscesisError,
};

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

    #[allow(dead_code)]
    fn get_root_verified(&self) -> Result<&ImmediateDef, AscesisError> {
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

    fn get_root(&self) -> Result<&ImmediateDef, AscesisError> {
        if let Some(ndx) = self.root {
            if let CesFileBlock::Imm(ref root) = self.blocks[ndx] {
                Ok(root)
            } else {
                unreachable!()
            }
        } else {
            Err(AscesisError::RootUnset)
        }
    }

    fn get_root_mut(&mut self) -> Result<&mut ImmediateDef, AscesisError> {
        if let Some(ndx) = self.root {
            if let CesFileBlock::Imm(ref mut root) = self.blocks[ndx] {
                Ok(root)
            } else {
                unreachable!()
            }
        } else {
            Err(AscesisError::RootUnset)
        }
    }

    pub fn get_vis_size<S: AsRef<str>>(&self, key: S) -> Option<u64> {
        let key = key.as_ref();

        for block in self.blocks.iter().rev() {
            if let CesFileBlock::Vis(blk) = block {
                let result = blk.get_size(key);
                if result.is_some() {
                    return result
                }
            }
        }
        None
    }

    pub fn get_vis_name<S: AsRef<str>>(&self, key: S) -> Option<&str> {
        let key = key.as_ref();

        for block in self.blocks.iter().rev() {
            if let CesFileBlock::Vis(blk) = block {
                let result = blk.get_name(key);
                if result.is_some() {
                    return result
                }
            }
        }
        None
    }

    pub fn get_nested_vis_size<I, S>(&self, subblock_keys: I, value_key: S) -> Option<u64>
    where
        I: IntoIterator + Clone,
        I::Item: AsRef<str>,
        S: AsRef<str>,
    {
        let value_key = value_key.as_ref();

        for block in self.blocks.iter().rev() {
            if let CesFileBlock::Vis(blk) = block {
                let result = blk.get_nested_size(subblock_keys.clone(), value_key);
                if result.is_some() {
                    return result
                }
            }
        }
        None
    }

    pub fn get_nested_vis_name<I, S>(&self, subblock_keys: I, value_key: S) -> Option<&str>
    where
        I: IntoIterator + Clone,
        I::Item: AsRef<str>,
        S: AsRef<str>,
    {
        let value_key = value_key.as_ref();

        for block in self.blocks.iter().rev() {
            if let CesFileBlock::Vis(blk) = block {
                let result = blk.get_nested_name(subblock_keys.clone(), value_key);
                if result.is_some() {
                    return result
                }
            }
        }
        None
    }

    pub fn get_sat_encoding(&self) -> Option<sat::Encoding> {
        for block in self.blocks.iter().rev() {
            if let CesFileBlock::SAT(blk) = block {
                if let Some(encoding) =
                    blk.get_name("encoding").or_else(|| blk.get_identifier("encoding"))
                {
                    match encoding {
                        "port-link" => return Some(sat::Encoding::PortLink),
                        "fork-join" => return Some(sat::Encoding::ForkJoin),
                        _ => {
                            error!("Invalid SAT encoding '{}'", encoding);
                            return None
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_sat_search(&self) -> Option<sat::Search> {
        for block in self.blocks.iter().rev() {
            if let CesFileBlock::SAT(blk) = block {
                if let Some(search) =
                    blk.get_name("search").or_else(|| blk.get_identifier("search"))
                {
                    match search {
                        "min" => return Some(sat::Search::MinSolutions),
                        "all" => return Some(sat::Search::AllSolutions),
                        _ => {
                            error!("Invalid SAT search '{}'", search);
                            return None
                        }
                    }
                }
            }
        }
        None
    }

    pub fn compile(&mut self, ctx: &ContextHandle) -> Result<(), Box<dyn Error>> {
        if let Some(encoding) = self.get_sat_encoding() {
            info!("Using encoding '{:?}'", encoding);
            ctx.lock().unwrap().set_encoding(encoding);
        }

        if let Some(search) = self.get_sat_search() {
            info!("Using '{:?}' search", search);
            ctx.lock().unwrap().set_search(search);
        }

        let root = self.get_root_mut()?;

        root.compile(ctx)
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
        self.get_root().ok().map(|root| root.name.as_str())
    }

    fn get_carrier_ids(&mut self) -> Vec<NodeID> {
        if let Ok(root) = self.get_root_mut() {
            if let Some(ref mut content) = root.content {
                content.get_carrier_ids()
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    fn get_causes_by_id(&self, id: NodeID) -> Option<&Vec<Vec<NodeID>>> {
        if let Ok(root) = self.get_root() {
            if let Some(ref content) = root.content {
                content.get_causes_by_id(id)
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }

    fn get_effects_by_id(&self, id: NodeID) -> Option<&Vec<Vec<NodeID>>> {
        if let Ok(root) = self.get_root() {
            if let Some(ref content) = root.content {
                content.get_effects_by_id(id)
            } else {
                panic!()
            }
        } else {
            panic!()
        }
    }
}

#[derive(Debug)]
pub enum CesFileBlock {
    Imm(ImmediateDef),
    Vis(PropBlock),
    SAT(PropBlock),
    Cap(CapacityBlock),
    Mul(MultiplierBlock),
    Inh(InhibitorBlock),
}

impl From<ImmediateDef> for CesFileBlock {
    fn from(imm: ImmediateDef) -> Self {
        CesFileBlock::Imm(imm)
    }
}

impl From<PropBlock> for CesFileBlock {
    fn from(props: PropBlock) -> Self {
        if let Some(selector) = props.get_selector() {
            match selector {
                PropSelector::Vis => CesFileBlock::Vis(props),
                PropSelector::SAT => CesFileBlock::SAT(props),
            }
        } else {
            panic!() // FIXME
        }
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
    name:    CesName,
    rex:     Rex,
    content: Option<PartialContent>,
}

impl ImmediateDef {
    pub fn new(name: CesName, rex: Rex) -> Self {
        ImmediateDef { name, rex, content: None }
    }

    pub fn compile(&mut self, ctx: &ContextHandle) -> Result<(), Box<dyn Error>> {
        let content = self.rex.compile_as_content(ctx)?;

        self.content = Some(content);

        Ok(())
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
