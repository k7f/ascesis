use std::{fmt, error::Error};
use crate::PropSelector;

pub(crate) type ParserError = lalrpop_util::ParseError<usize, String, String>;

#[derive(Clone, Debug)]
pub enum AscesisError {
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
    BlockSelectorMismatch(PropSelector, PropSelector),
    SizeLiteralOverflow,
    ExpectedSizeLiteral,
    ExpectedNameLiteral,
}

impl fmt::Display for AscesisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AscesisError::*;

        match self {
            ParsingRecovery(errors) => {
                for (num, err) in errors.iter().enumerate() {
                    let message = format!("{}", err);
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

        AscesisError::ParsingRecovery(vec![err])
    }
}

impl<L: Into<usize>, T: fmt::Display, E: Into<String>>
    From<Vec<lalrpop_util::ErrorRecovery<L, T, E>>> for AscesisError
{
    fn from(err: Vec<lalrpop_util::ErrorRecovery<L, T, E>>) -> Self {
        let err = err
            .into_iter()
            .map(|e| {
                e.error
                    .map_location(|l| l.into())
                    .map_token(|t| t.to_string())
                    .map_error(|e| e.into())
            })
            .collect();

        AscesisError::ParsingRecovery(err)
    }
}
