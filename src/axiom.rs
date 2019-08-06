use std::{fmt, str::FromStr};
use regex::Regex;
use crate::ascesis_parser::{
    CesFileBlockParser, ImmediateDefParser, CesInstanceParser, CapBlockParser, MulBlockParser,
    InhBlockParser, RexParser, ThinArrowRuleParser, FatArrowRuleParser, PolynomialParser,
};
use crate::{
    ParsingError, ParsingResult, AscesisError, CesFileBlock, ImmediateDef, CesInstance,
    CapacityBlock, MultiplierBlock, InhibitorBlock, Rex, ThinArrowRule, FatArrowRule, Polynomial,
};

#[derive(Clone, Debug)]
pub struct Axiom(String);

impl Axiom {
    pub fn from_known_symbol<S: AsRef<str>>(symbol: S) -> Option<Self> {
        let symbol = symbol.as_ref();

        match symbol {
            "CesFileBlock" | "ImmediateDef" | "CesInstance" | "CapBlock" | "MulBlock"
            | "InhBlock" | "Rex" | "ThinArrowRule" | "FatArrowRule" | "Polynomial" => {
                Some(Axiom(symbol.to_owned()))
            }
            _ => None,
        }
    }

    pub fn guess_from_phrase<S: AsRef<str>>(phrase: S) -> Self {
        lazy_static! {
            static ref IMM_RE: Regex = Regex::new(r"^ces\s+[[:alpha:]][[:word:]]*\s*\{").unwrap();
            static ref CAP_RE: Regex = Regex::new(r"^cap\s*\{").unwrap();
            static ref MUL_RE: Regex = Regex::new(r"^mul\s*\{").unwrap();
            static ref INH_RE: Regex = Regex::new(r"^inh\s*\{").unwrap();
            static ref TIN_RE: Regex = Regex::new(r"^[[:alpha:]][[:word:]]*\s*!\s*\(").unwrap();
            static ref IIN_RE: Regex =
                Regex::new(r"^[[:alpha:]][[:word:]]*\s*\(\s*\)\s*$").unwrap();
            static ref REX_RE: Regex = Regex::new(r"(\{|,|!|\(\s*\))").unwrap();
            static ref TAR_RE: Regex = Regex::new(r"(->|<-)").unwrap();
            static ref FAR_RE: Regex = Regex::new(r"(=>|<=)").unwrap();
        }

        let phrase = phrase.as_ref().trim();

        if IMM_RE.is_match(phrase) {
            Axiom("ImmediateDef".to_owned())
        } else if CAP_RE.is_match(phrase) {
            Axiom("CapBlock".to_owned())
        } else if MUL_RE.is_match(phrase) {
            Axiom("MulBlock".to_owned())
        } else if INH_RE.is_match(phrase) {
            Axiom("InhBlock".to_owned())
        } else if TIN_RE.is_match(phrase) || IIN_RE.is_match(phrase) {
            Axiom("CesInstance".to_owned())
        } else if REX_RE.is_match(phrase) {
            Axiom("Rex".to_owned())
        } else if TAR_RE.is_match(phrase) {
            Axiom("ThinArrowRule".to_owned())
        } else if FAR_RE.is_match(phrase) {
            Axiom("FatArrowRule".to_owned())
        } else {
            // FIXME into tests: `a(b)` is a Polynomial, `a()`,
            // `a(b,)` are instantiations.
            Axiom("Polynomial".to_owned())
        }
    }

    pub fn symbol(&self) -> &str {
        self.0.as_str()
    }

    pub fn parse<S: AsRef<str>>(&self, phrase: S) -> Result<Box<dyn FromPhrase>, AscesisError> {
        macro_rules! from_phrase_as {
            ($typ:ty, $phrase:expr) => {{
                let object: $typ = $phrase.parse()?;
                Ok(Box::new(object))
            }};
        }

        let phrase = phrase.as_ref();

        match self.0.as_str() {
            "CesFileBlock" => from_phrase_as!(CesFileBlock, phrase),
            "ImmediateDef" => from_phrase_as!(ImmediateDef, phrase),
            "CesInstance" => from_phrase_as!(CesInstance, phrase),
            "CapBlock" => from_phrase_as!(CapacityBlock, phrase),
            "MulBlock" => from_phrase_as!(MultiplierBlock, phrase),
            "InhBlock" => from_phrase_as!(InhibitorBlock, phrase),
            "Rex" => from_phrase_as!(Rex, phrase),
            "ThinArrowRule" => from_phrase_as!(ThinArrowRule, phrase),
            "FatArrowRule" => from_phrase_as!(FatArrowRule, phrase),
            "Polynomial" => from_phrase_as!(Polynomial, phrase),
            _ => Err(AscesisError::UnknownAxiom(self.0.clone())),
        }
    }
}

pub trait FromPhrase: fmt::Debug {
    fn from_phrase<S>(phrase: S) -> ParsingResult<Self>
    where
        S: AsRef<str>,
        Self: Sized;
}

macro_rules! impl_from_phrase_for {
    ($nt:ty, $parser:ty) => {
        impl FromPhrase for $nt {
            fn from_phrase<S: AsRef<str>>(phrase: S) -> ParsingResult<Self> {
                let phrase = phrase.as_ref();
                let mut errors = Vec::new();

                let result = <$parser>::new().parse(&mut errors, phrase).map_err(|err| {
                    err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned())
                })?;

                Ok(result)
            }
        }
    };
}

impl_from_phrase_for!(CesFileBlock, CesFileBlockParser);
impl_from_phrase_for!(ImmediateDef, ImmediateDefParser);
impl_from_phrase_for!(CesInstance, CesInstanceParser);
impl_from_phrase_for!(CapacityBlock, CapBlockParser);
impl_from_phrase_for!(MultiplierBlock, MulBlockParser);
impl_from_phrase_for!(InhibitorBlock, InhBlockParser);
impl_from_phrase_for!(Rex, RexParser);
impl_from_phrase_for!(ThinArrowRule, ThinArrowRuleParser);
impl_from_phrase_for!(FatArrowRule, FatArrowRuleParser);
impl_from_phrase_for!(Polynomial, PolynomialParser);

macro_rules! impl_from_str_for {
    ($nt:ty) => {
        impl FromStr for $nt {
            type Err = ParsingError;

            fn from_str(s: &str) -> ParsingResult<Self> {
                Self::from_phrase(s)
            }
        }
    };
}

impl_from_str_for!(CesFileBlock);
impl_from_str_for!(ImmediateDef);
impl_from_str_for!(CesInstance);
impl_from_str_for!(CapacityBlock);
impl_from_str_for!(MultiplierBlock);
impl_from_str_for!(InhibitorBlock);
impl_from_str_for!(Rex);
impl_from_str_for!(ThinArrowRule);
impl_from_str_for!(FatArrowRule);
impl_from_str_for!(Polynomial);
