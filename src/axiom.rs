use std::{fmt, str::FromStr};
use regex::Regex;
use crate::ascesis_parser::{
    CesFileParser, CesFileBlockParser, ImmediateDefParser, CesImmediateParser, CesInstanceParser,
    PropBlockParser, CapsBlockParser, UnboundedBlockParser, WeightsBlockParser, InhibitBlockParser,
    WeightlessBlockParser, RexParser, ThinArrowRuleParser, FatArrowRuleParser, PolynomialParser,
};
use crate::{
    CesFile, CesFileBlock, ImmediateDef, CesImmediate, CesInstance, PropBlock, CapacitiesBlock,
    UnboundedBlock, WeightsBlock, InhibitorsBlock, WeightlessBlock, Rex, ThinArrowRule,
    FatArrowRule, Polynomial, Lexer, AscesisError, AscesisErrorKind, error::ParserError,
};

#[derive(Clone, Debug)]
pub struct Axiom(String);

impl Axiom {
    pub fn from_known_symbol<S: AsRef<str>>(symbol: S) -> Option<Self> {
        let symbol = symbol.as_ref();

        match symbol {
            "CesFileBlock" | "ImmediateDef" | "CesImmediate" | "CesInstance" | "PropBlock"
            | "CapsBlock" | "UnboundedBlock" | "WeightsBlock" | "InhibitBlock"
            | "ActivateBlock" | "DropBlock" | "Rex" | "ThinArrowRule" | "FatArrowRule"
            | "Polynomial" => Some(Axiom(symbol.to_owned())),
            _ => None,
        }
    }

    pub fn guess_from_phrase<S: AsRef<str>>(phrase: S) -> Self {
        lazy_static! {
            static ref IMM_RE: Regex = Regex::new(r"^ces\s+[[:alpha:]][[:word:]]*\s*\{").unwrap();
            static ref VIS_RE: Regex = Regex::new(r"^vis\s*\{").unwrap();
            static ref SAT_RE: Regex = Regex::new(r"^sat\s*\{").unwrap();
            static ref CAPS_RE: Regex = Regex::new(r"^caps\s*\{").unwrap();
            static ref UNBOUNDED_RE: Regex = Regex::new(r"^unbounded\s*\{").unwrap();
            static ref WEIGHTS_RE: Regex = Regex::new(r"^weights\s*\{").unwrap();
            static ref INHIBIT_RE: Regex = Regex::new(r"^inhibit\s*\{").unwrap();
            static ref ACTIVATE_RE: Regex = Regex::new(r"^activate\s*\{").unwrap();
            static ref DROP_RE: Regex = Regex::new(r"^drop\s*\{").unwrap();
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
        } else if VIS_RE.is_match(phrase) || SAT_RE.is_match(phrase) {
            Axiom("PropBlock".to_owned())
        } else if CAPS_RE.is_match(phrase) {
            Axiom("CapsBlock".to_owned())
        } else if UNBOUNDED_RE.is_match(phrase) {
            Axiom("UnboundedBlock".to_owned())
        } else if WEIGHTS_RE.is_match(phrase) {
            Axiom("WeightsBlock".to_owned())
        } else if INHIBIT_RE.is_match(phrase) {
            Axiom("InhibitBlock".to_owned())
        } else if ACTIVATE_RE.is_match(phrase) {
            Axiom("ActivateBlock".to_owned())
        } else if DROP_RE.is_match(phrase) {
            Axiom("DropBlock".to_owned())
        } else if IIN_RE.is_match(phrase) {
            Axiom("CesImmediate".to_owned())
        } else if TIN_RE.is_match(phrase) {
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
                let object: $typ = $phrase.parse().map_err(AscesisErrorKind::from)?;
                Ok(Box::new(object))
            }};
        }

        let phrase = phrase.as_ref();

        match self.0.as_str() {
            "CesFileBlock" => from_phrase_as!(CesFileBlock, phrase),
            "ImmediateDef" => from_phrase_as!(ImmediateDef, phrase),
            "CesImmediate" => from_phrase_as!(CesImmediate, phrase),
            "CesInstance" => from_phrase_as!(CesInstance, phrase),
            "PropBlock" => from_phrase_as!(PropBlock, phrase),
            "CapsBlock" => from_phrase_as!(CapacitiesBlock, phrase),
            "UnboundedBlock" => from_phrase_as!(UnboundedBlock, phrase),
            "WeightsBlock" => from_phrase_as!(WeightsBlock, phrase),
            "InhibitBlock" => from_phrase_as!(InhibitorsBlock, phrase),
            "ActivateBlock" => from_phrase_as!(WeightlessBlock, phrase),
            "DropBlock" => from_phrase_as!(WeightlessBlock, phrase),
            "Rex" => from_phrase_as!(Rex, phrase),
            "ThinArrowRule" => from_phrase_as!(ThinArrowRule, phrase),
            "FatArrowRule" => from_phrase_as!(FatArrowRule, phrase),
            "Polynomial" => from_phrase_as!(Polynomial, phrase),
            symbol => Err(AscesisErrorKind::AxiomUnknown(symbol.into()).with_script(phrase)),
        }
    }
}

pub trait FromPhrase: fmt::Debug {
    fn from_phrase<S>(phrase: S) -> Result<Self, ParserError>
    where
        S: AsRef<str>,
        Self: Sized;
}

macro_rules! impl_from_phrase_for {
    ($nt:ty, $parser:ty) => {
        impl FromPhrase for $nt {
            fn from_phrase<S: AsRef<str>>(phrase: S) -> Result<Self, ParserError> {
                let phrase = phrase.as_ref();
                let mut errors = Vec::new();
                let lexer = Lexer::new(phrase);

                let result = <$parser>::new().parse(&mut errors, lexer).map_err(|err| {
                    err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned())
                })?;

                Ok(result)
            }
        }
    };
}

impl_from_phrase_for!(CesFile, CesFileParser);
impl_from_phrase_for!(CesFileBlock, CesFileBlockParser);
impl_from_phrase_for!(ImmediateDef, ImmediateDefParser);
impl_from_phrase_for!(CesImmediate, CesImmediateParser);
impl_from_phrase_for!(CesInstance, CesInstanceParser);
impl_from_phrase_for!(PropBlock, PropBlockParser);
impl_from_phrase_for!(CapacitiesBlock, CapsBlockParser);
impl_from_phrase_for!(UnboundedBlock, UnboundedBlockParser);
impl_from_phrase_for!(WeightsBlock, WeightsBlockParser);
impl_from_phrase_for!(InhibitorsBlock, InhibitBlockParser);
impl_from_phrase_for!(WeightlessBlock, WeightlessBlockParser);
impl_from_phrase_for!(Rex, RexParser);
impl_from_phrase_for!(ThinArrowRule, ThinArrowRuleParser);
impl_from_phrase_for!(FatArrowRule, FatArrowRuleParser);
impl_from_phrase_for!(Polynomial, PolynomialParser);

macro_rules! impl_from_str_for {
    ($nt:ty) => {
        impl FromStr for $nt {
            type Err = ParserError;

            fn from_str(s: &str) -> Result<Self, ParserError> {
                Self::from_phrase(s)
            }
        }
    };
}

impl_from_str_for!(CesFile);
impl_from_str_for!(CesFileBlock);
impl_from_str_for!(ImmediateDef);
impl_from_str_for!(CesImmediate);
impl_from_str_for!(CesInstance);
impl_from_str_for!(PropBlock);
impl_from_str_for!(CapacitiesBlock);
impl_from_str_for!(UnboundedBlock);
impl_from_str_for!(WeightsBlock);
impl_from_str_for!(InhibitorsBlock);
impl_from_str_for!(WeightlessBlock);
impl_from_str_for!(Rex);
impl_from_str_for!(ThinArrowRule);
impl_from_str_for!(FatArrowRule);
impl_from_str_for!(Polynomial);
