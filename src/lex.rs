use std::{fmt, convert::TryFrom, str::FromStr, error::Error};
use enquote::unquote;
use crate::AscesisError;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Literal {
    Size(u64),
    Omega,
    Theta,
    Name(String),
}

impl Literal {
    pub(crate) fn from_digits(digits: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Literal::Size(u64::from_str(digits)?))
    }

    #[inline]
    pub(crate) fn omega() -> Self {
        Literal::Omega
    }

    #[inline]
    pub(crate) fn theta() -> Self {
        Literal::Theta
    }

    pub(crate) fn from_quoted_str(quoted: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Literal::Name(unquote(quoted)?))
    }
}

impl TryFrom<Literal> for u64 {
    type Error = AscesisError;

    fn try_from(lit: Literal) -> Result<Self, Self::Error> {
        if let Literal::Size(size) = lit {
            Ok(size)
        } else {
            Err(AscesisError::ExpectedSizeLiteral)
        }
    }
}

impl TryFrom<Literal> for String {
    type Error = &'static str;

    fn try_from(lit: Literal) -> Result<Self, Self::Error> {
        if let Literal::Name(name) = lit {
            Ok(name)
        } else {
            Err("Bad literal, not a string")
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    ThinTx,
    ThinRx,
    FatTx,
    FatRx,
    FatDx,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinOp::Add => '+'.fmt(f),
            BinOp::ThinTx => "->".fmt(f),
            BinOp::ThinRx => "<-".fmt(f),
            BinOp::FatTx => "=>".fmt(f),
            BinOp::FatRx => "<=".fmt(f),
            BinOp::FatDx => "<=>".fmt(f),
        }
    }
}
