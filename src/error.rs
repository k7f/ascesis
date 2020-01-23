use std::{fmt, error::Error};
use crate::PropSelector;

pub type ParsingError = lalrpop_util::ParseError<usize, String, String>;
pub type ParsingResult<T> = Result<T, ParsingError>;

#[derive(Clone, Debug)]
pub enum AscesisError {
    ParsingError(ParsingError),
    ParsingRecovery,
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
    BlockSelectorMismatch(PropSelector, PropSelector),
    SizeLiteralOverflow,
    ExpectedSizeLiteral,
    ExpectedNameLiteral,
}

impl fmt::Display for AscesisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AscesisError::*;

        match self {
            ParsingError(err) => {
                let message = format!("{}", err);
                let mut lines = message.lines();

                if let Some(line) = lines.next() {
                    writeln!(f, "{}", line)?;
                }

                for line in lines {
                    writeln!(f, "\t{}", line)?;
                }

                Ok(())
            }
            ParsingRecovery => write!(f, "Recovering from ascesis parsing error"),
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
            BlockSelectorMismatch(expected, actual) => {
                write!(f, "Expecting {} selector, got {}", expected, actual)
            }
            SizeLiteralOverflow => write!(f, "Size literal overflow"),
            ExpectedSizeLiteral => write!(f, "Bad literal, not a size"),
            ExpectedNameLiteral => write!(f, "Bad literal, not a name"),
        }
    }
}

impl Error for AscesisError {}

impl<L: Into<usize>, T: fmt::Display, E: Into<String>> From<lalrpop_util::ParseError<L, T, E>>
    for AscesisError
{
    fn from(err: lalrpop_util::ParseError<L, T, E>) -> Self {
        let err =
            err.map_location(|l| l.into()).map_token(|t| t.to_string()).map_error(|e| e.into());

        AscesisError::ParsingError(err)
    }
}
