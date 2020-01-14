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

    fn script_is_acceptable(&self, script: &str) -> bool {
        let mut words = script.split_whitespace();

        if let Some(word) = words.next() {
            match word {
                "ces" => true,
                _ => {
                    if word.contains('{') {
                        word.trim_start_matches(|c: char| c.is_ascii_alphabetic()).starts_with('{')
                    } else if word.trim_start_matches(|c: char| c.is_ascii_alphabetic()).is_empty()
                    {
                        if let Some(word) = words.next() {
                            word.starts_with('{')
                        } else {
                            // Script is a single token, maybe a keyword, but not a block.
                            false
                        }
                    } else {
                        // Script is a single word, which is not a block.
                        false
                    }
                }
            }
        } else {
            // Script is empty.
            false
        }
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
