use std::{
    path::{Path, PathBuf},
    error::Error,
};
use aces::{ContextHandle, Content, ContentFormat, CompilableMut};
use crate::CesFile;

#[derive(Clone, Default, Debug)]
pub struct AscesisFormat {
    path: Option<PathBuf>,
}

impl AscesisFormat {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();

        AscesisFormat { path: Some(path) }
    }
}

impl ContentFormat for AscesisFormat {
    fn expected_extensions(&self) -> &[&str] {
        &["ces"]
    }

    fn script_is_acceptable(&self, _script: &str) -> bool {
        true // FIXME
    }

    fn script_to_content(
        &self,
        ctx: &ContextHandle,
        script: &str,
    ) -> Result<Box<dyn Content>, Box<dyn Error>> {
        let mut ces_file = CesFile::from_script(script)?;
        ces_file.set_root_name("Main")?;
        if let Some(title) = ces_file.get_vis_name("title") {
            info!("Using '{}' as the root structure: \"{}\"", ces_file.get_name().unwrap(), title);
        } else {
            info!("Using '{}' as the root structure", ces_file.get_name().unwrap());
        }

        ces_file.compile_mut(ctx)?;
        debug!("{:?}", ces_file);

        Ok(ces_file.into())
    }
}
