use std::{fmt, error::Error};

pub type ParsingError = lalrpop_util::ParseError<usize, String, String>;
pub type ParsingResult<T> = Result<T, ParsingError>;

#[derive(Clone, Debug)]
pub enum AscesisError {
    ParsingError(ParsingError),
    UnknownAxiom(String),
}

impl Error for AscesisError {
    fn description(&self) -> &str {
        use AscesisError::*;

        match self {
            ParsingError(_) => "ascesis parsing error",
            UnknownAxiom(_) => "unknown axiom",
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
            UnknownAxiom(symbol) => write!(f, "Unknown axiom '{}'", symbol),
        }
    }
}

impl From<ParsingError> for AscesisError {
    fn from(err: ParsingError) -> Self {
        AscesisError::ParsingError(err)
    }
}
