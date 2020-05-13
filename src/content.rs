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
        let mut script = script.trim_start();

        while script.starts_with("//") {
            if let Some(tail) = script.splitn(2, '\n').nth(1) {
                script = tail.trim_start();
            } else {
                // Script contains nothing but comments.
                return false
            }
        }

        let mut words = script.split_whitespace();

        if let Some(word) = words.next() {
            match word {
                "ces" => true,
                _ => {
                    if word.contains('{') {
                        // Script starts with a word containing left brace.
                        // Does this word resemble a prefix of an anonymous block?
                        word.trim_start_matches(|c: char| c.is_ascii_alphabetic()).starts_with('{')
                    } else if word.trim_start_matches(|c: char| c.is_ascii_alphabetic()).is_empty()
                    {
                        if let Some(word) = words.next() {
                            if word.starts_with('{') {
                                // Script starts with two words resembling
                                // a prefix of an anonymous block.
                                true
                            } else if word.contains('{') {
                                // Second word contains left brace.
                                // Does it all resemble a prefix of a named block?
                                word.trim_start_matches(|c: char| c.is_ascii_alphabetic())
                                    .starts_with('{')
                            } else if word
                                .trim_start_matches(|c: char| c.is_ascii_alphabetic())
                                .is_empty()
                            {
                                if let Some(word) = words.next() {
                                    // Script starts with three words.
                                    // Does it all resemble a prefix of a named block?
                                    word.starts_with('{')
                                } else {
                                    // Script consists of two tokens and isn't a block.
                                    false
                                }
                            } else {
                                // Script starts with two words where the second word
                                // isn't an identifier.
                                false
                            }
                        } else {
                            // Script is a single token, maybe a keyword, but not a block.
                            false
                        }
                    } else {
                        // Script starts with a word, but not a keyword.
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
        root_name: Option<&str>,
    ) -> Result<Box<dyn Content>, Box<dyn Error>> {
        let mut ces_file = CesFile::from_script(script)?;

        if let Some(root_name) = root_name {
            ces_file.set_root_name(root_name)?;

            if let Some(title) = ces_file.get_vis_name("title") {
                info!(
                    "Using '{}' as the root structure: \"{}\"",
                    ces_file.get_name().unwrap(),
                    title
                );
            } else {
                info!("Using '{}' as the root structure", ces_file.get_name().unwrap());
            }

            ces_file.compile_mut(ctx)?;
            debug!("{:?}", ces_file);
        }

        Ok(ces_file.into())
    }
}
