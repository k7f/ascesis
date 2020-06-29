use std::{fmt, convert::TryFrom, str::FromStr};
use logos::Logos;
use enquote::unquote;
use crate::{Weight, AscesisError, AscesisErrorKind};

#[derive(Clone, Copy, PartialEq, Logos, Debug)]
pub enum Token<'input> {
    #[error]
    Error,
    #[regex(r"\p{White_Space}", logos::skip)]
    WhiteSpace,
    // FIXME trim
    #[regex(r"///.*\n", |lex| lex.slice().strip_prefix("///").unwrap_or(""))]
    DocComment(&'input str),
    #[regex(r"//.*\n", logos::skip)]
    Comment,
    #[regex(r"[A-Za-z_][A-Za-z0-9_-]*", |lex| lex.slice())]
    Identifier(&'input str),
    #[regex(r"[0-9]+", |lex| lex.slice())]
    LiteralFiniteSize(&'input str),
    #[regex(r#""[^"]*""#, |lex| lex.slice())]
    LiteralName(&'input str),
    #[token("Ω")]
    #[token("ω")]
    Omega,
    #[token("Θ")]
    #[token("θ")]
    Theta,
    #[token(";")]
    Semicolon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token("{")]
    OpenCurly,
    #[token("}")]
    CloseCurly,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("+")]
    Add,
    #[token("->")]
    ThinArrow,
    #[token("<-")]
    ThinBackArrow,
    #[token("=>")]
    FatArrow,
    #[token("<=")]
    FatBackArrow,
    #[token("<=>")]
    FatTwowayArrow,
    #[token("!")]
    Bang,
    #[token("ces")]
    Ces,
    #[token("vis")]
    Vis,
    #[token("sat")]
    Sat,
    #[token("caps")]
    Caps,
    #[token("unbounded")]
    Unbounded,
    #[token("weights")]
    Weights,
    #[token("inhibit")]
    Inhibit,
    #[token("activate")]
    Activate,
    #[token("drop")]
    Drop,
}

impl<'input> fmt::Display for Token<'input> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Token::*;

        match self {
            Error => write!(f, "<error>"),
            WhiteSpace => write!(f, "<white-space>"),
            DocComment(_) => write!(f, "<doc-comment>"),
            Comment => write!(f, "<comment>"),
            Identifier(id) => write!(f, "{}", id),
            LiteralFiniteSize(s) => write!(f, "{}", s),
            LiteralName(s) => write!(f, "\"{}\"", s),
            Omega => write!(f, "ω"),
            Theta => write!(f, "θ"),
            Semicolon => write!(f, ";"),
            Comma => write!(f, ","),
            Dot => write!(f, "."),
            Colon => write!(f, ":"),
            OpenCurly => write!(f, "{{"),
            CloseCurly => write!(f, "}}"),
            OpenParen => write!(f, "("),
            CloseParen => write!(f, "("),
            OpenBracket => write!(f, "["),
            CloseBracket => write!(f, "]"),
            Add => write!(f, "+"),
            ThinArrow => write!(f, "->"),
            ThinBackArrow => write!(f, "<-"),
            FatArrow => write!(f, "=>"),
            FatBackArrow => write!(f, "<="),
            FatTwowayArrow => write!(f, "<=>"),
            Bang => write!(f, "!"),
            Ces => write!(f, "ces"),
            Vis => write!(f, "vis"),
            Sat => write!(f, "sat"),
            Caps => write!(f, "caps"),
            Unbounded => write!(f, "unbounded"),
            Weights => write!(f, "weights"),
            Inhibit => write!(f, "inhibit"),
            Activate => write!(f, "activate"),
            Drop => write!(f, "drop"),
        }
    }
}

impl<'input> From<Token<'input>> for String {
    fn from(token: Token<'input>) -> Self {
        use Token::*;

        match token {
            DocComment(s) | Identifier(s) | LiteralFiniteSize(s) | LiteralName(s) => s.into(),
            _ => format!("{}", token),
        }
    }
}

pub struct Lexer<'input>(logos::Lexer<'input, Token<'input>>);

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Lexer(Token::lexer(input))
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, Token<'input>, usize), AscesisError>;

    fn next(&mut self) -> Option<Self::Item> {
        let lexer = &mut self.0;

        lexer.next().map(|token| {
            let span = lexer.span();

            match token {
                Token::Error => Err(AscesisErrorKind::LexingFailure(lexer.slice().into(), span)
                    .with_script(lexer.source())),
                _ => Ok((span.start, token, span.end)),
            }
        })
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Literal {
    Size(u64),
    Omega,
    Theta,
    Name(String),
}

impl Literal {
    pub(crate) fn from_digits(digits: &str) -> Result<Self, AscesisError> {
        Ok(u64::from_str(digits).map(Literal::Size).map_err(Into::<AscesisErrorKind>::into)?)
    }

    #[inline]
    pub(crate) fn omega() -> Self {
        Literal::Omega
    }

    #[inline]
    pub(crate) fn theta() -> Self {
        Literal::Theta
    }

    pub(crate) fn from_quoted_str(quoted: &str) -> Result<Self, AscesisError> {
        Ok(unquote(quoted)
            .map(Literal::Name)
           // FIXME (replace enquote?)
            .map_err(|_| AscesisErrorKind::EnquoteFailure("Quoted string is invalid".into()))?)
    }
}

impl TryFrom<Literal> for u64 {
    type Error = AscesisError;

    fn try_from(lit: Literal) -> Result<Self, Self::Error> {
        if let Literal::Size(size) = lit {
            Ok(size)
        } else {
            Err(AscesisErrorKind::ExpectedSizeLiteral.into())
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

impl TryFrom<Literal> for Weight {
    type Error = AscesisError;

    fn try_from(lit: Literal) -> Result<Self, Self::Error> {
        match lit {
            Literal::Size(sz) => {
                Weight::finite(sz).ok_or_else(|| AscesisErrorKind::SizeLiteralOverflow.into())
            }
            Literal::Omega => Ok(Weight::omega()),
            _ => Err(AscesisErrorKind::ExpectedSizeLiteral.into()),
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
