use std::{fmt, error::Error};
use crate::{PropSelector, ascesis_parser::Token};

pub(crate) type ParserError = lalrpop_util::ParseError<usize, String, AscesisError>;
pub(crate) type RawParserError<'input> =
    lalrpop_util::ParseError<usize, Token<'input>, AscesisError>;
pub(crate) type RawParserRecovery<'input> =
    lalrpop_util::ErrorRecovery<usize, Token<'input>, AscesisError>;

impl From<AscesisError> for ParserError {
    fn from(error: AscesisError) -> Self {
        ParserError::User { error }
    }
}

impl<'input> From<AscesisError> for RawParserError<'input> {
    fn from(error: AscesisError) -> Self {
        RawParserError::User { error }
    }
}

fn format_location(mut pos: usize, script: &str) -> String {
    let mut num_lines = 0;

    for line in script.lines() {
        match pos.checked_sub(line.len() + 1) {
            Some(p) => pos = p,
            None => break,
        }
        num_lines += 1;
    }

    format!("[{}:{}]", num_lines + 1, pos + 1)
}

fn display_parsing_recovery(
    errors: &[ParserError],
    script: Option<&str>,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    for (num, err) in errors.iter().enumerate() {
        let message = if let Some(script) = script {
            format!("{}", err.clone().map_location(|pos| format_location(pos, script)))
        } else {
            format!("{}", err)
        };
        let mut lines = message.lines();

        if let Some(line) = lines.next() {
            if num > 0 {
                write!(f, "\nerror: {}", line)?;
            } else {
                write!(f, "{}", line)?;
            }

            for line in lines {
                write!(f, "\n\t{}", line)?;
            }
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub enum AscesisErrorKind {
    ParsingRecovery(Vec<ParserError>),
    ParsingFailure,
    AxiomUnknown(String),
    RootUnset,
    RootMissing(String),
    RootRedefined(String),
    RootBlockMismatch,
    RootBlockMissing,
    RootUnresolvable,
    ScriptUncompiled,
    UnexpectedDependency(String),
    InvalidAST,
    FatLeak,
    MissingPropSelector,
    InvalidPropSelector(String),
    InvalidPropType(PropSelector, String),
    InvalidPropValue(PropSelector, String, String),
    InvalidPropValueType(String),
    BlockSelectorMismatch(PropSelector, PropSelector),
    SizeLiteralOverflow,
    ExpectedSizeLiteral,
    ExpectedNameLiteral,
}

impl fmt::Display for AscesisErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AscesisErrorKind::*;

        match self {
            ParsingRecovery(ref errors) => display_parsing_recovery(errors, None, f),
            ParsingFailure => write!(f, "Recovering from ascesis parsing errors"),
            AxiomUnknown(symbol) => write!(f, "Unknown axiom '{}'", symbol),
            RootUnset => write!(f, "Undeclared root structure"),
            RootMissing(name) => write!(f, "Missing root structure '{}'", name),
            RootRedefined(name) => write!(f, "Redefined root structure '{}'", name),
            RootBlockMismatch => write!(f, "Root block mismatch"),
            RootBlockMissing => write!(f, "Root block missing"),
            RootUnresolvable => write!(f, "Root contains instances without known definitions"),
            ScriptUncompiled => write!(f, "Script uncompiled"),
            UnexpectedDependency(name) => write!(f, "Unexpected uncompiled dependency '{}'", name),
            InvalidAST => write!(f, "Invalid AST"),
            FatLeak => write!(f, "Fat arrow rule leaked through FIT transformation"),
            MissingPropSelector => write!(f, "Property block without selector"),
            InvalidPropSelector(name) => write!(f, "Invalid block selector '{}'", name),
            InvalidPropType(selector, prop) => write!(f, "Invalid {} {} type", selector, prop),
            InvalidPropValue(selector, prop, value) => {
                write!(f, "Invalid {} {} '{}'", selector, prop, value)
            }
            InvalidPropValueType(given) => write!(f, "Property value type not a {}", given),
            BlockSelectorMismatch(expected, actual) => {
                write!(f, "Expecting {} selector, got {}", expected, actual)
            }
            SizeLiteralOverflow => write!(f, "Size literal overflow"),
            ExpectedSizeLiteral => write!(f, "Bad literal, not a size"),
            ExpectedNameLiteral => write!(f, "Bad literal, not a name"),
        }
    }
}

impl AscesisErrorKind {
    pub fn with_script<S: AsRef<str>>(self, script: S) -> AscesisError {
        AscesisError { script: Some(script.as_ref().to_owned()), kind: self }
    }
}

impl From<ParserError> for AscesisErrorKind {
    fn from(err: ParserError) -> Self {
        AscesisErrorKind::ParsingRecovery(vec![err])
    }
}

impl<'input> From<RawParserError<'input>> for AscesisErrorKind {
    fn from(err: RawParserError<'input>) -> Self {
        AscesisErrorKind::ParsingRecovery(vec![err.map_token(|t| t.to_string())])
    }
}

impl<'input> From<Vec<RawParserRecovery<'input>>> for AscesisErrorKind {
    fn from(err: Vec<RawParserRecovery<'input>>) -> Self {
        let err = err.into_iter().map(|e| e.error.map_token(|t| t.to_string())).collect();

        AscesisErrorKind::ParsingRecovery(err)
    }
}

#[derive(Clone, Debug)]
pub struct AscesisError {
    script: Option<String>,
    kind:   AscesisErrorKind,
}

impl From<AscesisErrorKind> for AscesisError {
    fn from(kind: AscesisErrorKind) -> Self {
        AscesisError { script: None, kind }
    }
}

impl fmt::Display for AscesisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AscesisErrorKind::*;

        if let Some(ref script) = self.script {
            match self.kind {
                ParsingRecovery(ref errors) => display_parsing_recovery(errors, Some(script), f),
                ref kind => kind.fmt(f),
            }
        } else {
            self.kind.fmt(f)
        }
    }
}

impl Error for AscesisError {}
