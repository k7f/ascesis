use std::{fmt, error::Error};

pub type ParsingError = lalrpop_util::ParseError<usize, String, String>;
pub type ParsingResult<T> = Result<T, ParsingError>;

#[derive(Clone, Debug)]
pub enum AscesisError {
    ParsingError(ParsingError),
    AxiomUnknown(String),
    RootUnset,
    RootMissing(String),
    RootRedefined(String),
    RootBlockMismatch,
    RootBlockMissing,
    InvalidAST,
    FatLeak,
}

impl Error for AscesisError {
    fn description(&self) -> &str {
        use AscesisError::*;

        match self {
            ParsingError(_) => "ascesis parsing error",
            AxiomUnknown(_) => "unknown axiom",
            RootUnset => "unset root structure",
            RootMissing(_) => "missing root structure",
            RootRedefined(_) => "redefined root structure",
            RootBlockMismatch => "root block mismatch",
            RootBlockMissing => "root block missing",
            InvalidAST => "invalid AST",
            FatLeak => "fat arrow rule leaked through FIT transformation",
        }
    }
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
            AxiomUnknown(symbol) => write!(f, "Unknown axiom '{}'", symbol),
            RootUnset => write!(f, "Undeclared root structure"),
            RootMissing(name) => write!(f, "Missing root structure '{}'", name),
            RootRedefined(name) => write!(f, "Redefined root structure '{}'", name),
            RootBlockMismatch => write!(f, "Root block mismatch"),
            RootBlockMissing => write!(f, "Root block missing"),
            InvalidAST => write!(f, "Invalid AST"),
            FatLeak => write!(f, "Fat arrow rule leaked through FIT transformation"),
        }
    }
}

impl From<ParsingError> for AscesisError {
    fn from(err: ParsingError) -> Self {
        AscesisError::ParsingError(err)
    }
}
