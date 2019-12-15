use std::{ops::Deref, fmt, error::Error};
use log::Level::Debug;
use aces::{
    Content, PartialContent, Compilable, CompilableMut, CompilableAsContent,
    CompilableAsDependency, ContextHandle, NodeID, sat,
};
use crate::{
    PropBlock, PropSelector, CapacityBlock, MultiplicityBlock, InhibitorBlock, Rex, AscesisError,
};

#[derive(Default, Debug)]
pub struct CesFile {
    script:  Option<String>,
    blocks:  Vec<CesFileBlock>,
    root:    Option<usize>,
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
                        return Err(AscesisError::RootRedefined(root_name.to_owned()).into())
                    }
                }
            }
        }

        if self.root.is_some() {
            Ok(())
        } else {
            Err(AscesisError::RootMissing(root_name.to_owned()).into())
        }
    }

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

    fn get_content(&self) -> Result<&PartialContent, AscesisError> {
        if let Some(ref content) = self.content {
            Ok(content)
        } else {
            self.get_root_verified().and(Err(AscesisError::ScriptUncompiled))
        }
    }

    fn get_content_mut(&mut self) -> Result<&mut PartialContent, AscesisError> {
        if let Some(ref mut content) = self.content {
            Ok(content)
        } else {
            self.get_root_verified().and(Err(AscesisError::ScriptUncompiled))
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

    pub fn get_sat_encoding(&self) -> Result<Option<sat::Encoding>, AscesisError> {
        for block in self.blocks.iter().rev() {
            if let CesFileBlock::SAT(blk) = block {
                if let Some(encoding) = blk.get_sat_encoding()? {
                    return Ok(Some(encoding))
                }
            }
        }

        Ok(None)
    }

    pub fn get_sat_search(&self) -> Result<Option<sat::Search>, AscesisError> {
        for block in self.blocks.iter().rev() {
            if let CesFileBlock::SAT(blk) = block {
                if let Some(search) = blk.get_sat_search()? {
                    return Ok(Some(search))
                }
            }
        }

        Ok(None)
    }
}

impl CompilableMut for CesFile {
    fn compile_mut(&mut self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        info!("Start compiling...");

        // First pass: compile all property blocks.

        for block in self.blocks.iter().rev() {
            match block {
                CesFileBlock::SAT(blk) | CesFileBlock::Vis(blk) => {
                    blk.compile(ctx)?;
                }
                _ => {}
            }
        }

        // Second pass: compile all structural blocks having no dependencies.

        for block in self.blocks.iter_mut() {
            match block {
                CesFileBlock::Imm(ref mut imm) => {
                    imm.compile(ctx)?;
                }
                CesFileBlock::Cap(ref cap) => {
                    cap.compile(ctx)?;
                }
                CesFileBlock::Mul(ref mul) => {
                    mul.compile(ctx)?;
                }
                CesFileBlock::Inh(ref inh) => {
                    inh.compile(ctx)?;
                }
                _ => {} // FIXME
            }
        }

        loop {
            // Repeat compiling all resolvable uncompiled Imm blocks
            // until reaching a fix point.

            let mut made_progress = false;

            for block in self.blocks.iter_mut() {
                if let CesFileBlock::Imm(ref mut imm) = block {
                    if !imm.is_compiled(ctx) && imm.compile(ctx)? {
                        made_progress = true;
                    }
                }
            }

            if !made_progress {
                break
            }
        }

        let root = self.get_root()?;

        if root.is_compiled(ctx) {
            let content = root.get_compiled_content(ctx)?;

            self.content = Some(content);

            Ok(true)
        } else {
            Err(AscesisError::RootUnresolvable.into())
        }
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
        let content = self.get_content_mut().unwrap();

        content.get_carrier_ids()
    }

    fn get_causes_by_id(&self, id: NodeID) -> Option<&Vec<Vec<NodeID>>> {
        let content = self.get_content().unwrap();

        content.get_causes_by_id(id)
    }

    fn get_effects_by_id(&self, id: NodeID) -> Option<&Vec<Vec<NodeID>>> {
        let content = self.get_content().unwrap();

        content.get_effects_by_id(id)
    }
}

#[derive(Debug)]
pub enum CesFileBlock {
    Imm(ImmediateDef),
    Vis(PropBlock),
    SAT(PropBlock),
    Cap(CapacityBlock),
    Mul(MultiplicityBlock),
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

impl From<MultiplicityBlock> for CesFileBlock {
    fn from(mul: MultiplicityBlock) -> Self {
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

impl AsRef<str> for CesName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for CesName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
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
        println!("ImmediateDef of '{}': {:?}", name, rex);
        ImmediateDef { name, rex }
    }

    pub(crate) fn is_compiled(&self, ctx: &ContextHandle) -> bool {
        ctx.lock().unwrap().has_content(&self.name)
    }
}

impl Compilable for ImmediateDef {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        if log_enabled!(Debug) {
            if let Some(dep_name) = self.compile_as_dependency(ctx)? {
                debug!("Still not compiled ImmediateDef of '{}': missing {}", self.name, dep_name);

                Ok(false)
            } else {
                let content = self.get_compiled_content(ctx)?;

                debug!("OK compiled ImmediateDef of '{}': {:?}", self.name, content);

                Ok(true)
            }
        } else {
            Ok(self.compile_as_dependency(ctx)?.is_none())
        }
    }
}

impl CompilableAsContent for ImmediateDef {
    fn get_compiled_content(&self, ctx: &ContextHandle) -> Result<PartialContent, Box<dyn Error>> {
        if let Some(content) = ctx.lock().unwrap().get_content(&self.name) {
            Ok(content.clone())
        } else if let Some(dep_name) = self.compile_as_dependency(ctx)? {
            Err(AscesisError::UnexpectedDependency(dep_name).into())
        } else if let Some(content) = ctx.lock().unwrap().get_content(&self.name) {
            Ok(content.clone())
        } else {
            panic!()
        }
    }

    fn check_dependencies(&self, ctx: &ContextHandle) -> Option<String> {
        self.rex.check_dependencies(ctx)
    }
}

impl CompilableAsDependency for ImmediateDef {
    fn compile_as_dependency(&self, ctx: &ContextHandle) -> Result<Option<String>, Box<dyn Error>> {
        if let Some(dep_name) = self.rex.check_dependencies(ctx) {
            Ok(Some(dep_name))
        } else {
            let content = self.rex.get_compiled_content(ctx)?;
            let mut ctx = ctx.lock().unwrap();

            ctx.add_content(&self.name, content);

            Ok(None)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CesInstance {
    pub(crate) name: CesName,
    pub(crate) args: Vec<String>,
}

impl CesInstance {
    pub(crate) fn new(name: CesName) -> Self {
        println!("CesInstance of '{}'", name);
        CesInstance { name, args: Vec::new() }
    }

    pub(crate) fn with_args(mut self, mut args: Vec<String>) -> Self {
        self.args.append(&mut args);
        self
    }
}
