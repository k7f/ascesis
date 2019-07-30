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

    pub fn guess_from_spec<S: AsRef<str>>(spec: S) -> Self {
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

        let spec = spec.as_ref().trim();

        if IMM_RE.is_match(spec) {
            Axiom("ImmediateDef".to_owned())
        } else if CAP_RE.is_match(spec) {
            Axiom("CapBlock".to_owned())
        } else if MUL_RE.is_match(spec) {
            Axiom("MulBlock".to_owned())
        } else if INH_RE.is_match(spec) {
            Axiom("InhBlock".to_owned())
        } else if TIN_RE.is_match(spec) || IIN_RE.is_match(spec) {
            Axiom("CesInstance".to_owned())
        } else if REX_RE.is_match(spec) {
            Axiom("Rex".to_owned())
        } else if TAR_RE.is_match(spec) {
            Axiom("ThinArrowRule".to_owned())
        } else if FAR_RE.is_match(spec) {
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

    pub fn parse<S: AsRef<str>>(&self, spec: S) -> Result<Box<dyn FromSpec>, AscesisError> {
        macro_rules! from_spec_as {
            ($typ:ty, $spec:expr) => {{
                let object: $typ = $spec.parse()?;
                Ok(Box::new(object))
            }};
        }

        let spec = spec.as_ref();

        match self.0.as_str() {
            "CesFileBlock" => from_spec_as!(CesFileBlock, spec),
            "ImmediateDef" => from_spec_as!(ImmediateDef, spec),
            "CesInstance" => from_spec_as!(CesInstance, spec),
            "CapBlock" => from_spec_as!(CapacityBlock, spec),
            "MulBlock" => from_spec_as!(MultiplierBlock, spec),
            "InhBlock" => from_spec_as!(InhibitorBlock, spec),
            "Rex" => from_spec_as!(Rex, spec),
            "ThinArrowRule" => from_spec_as!(ThinArrowRule, spec),
            "FatArrowRule" => from_spec_as!(FatArrowRule, spec),
            "Polynomial" => from_spec_as!(Polynomial, spec),
            _ => Err(AscesisError::UnknownAxiom(self.0.clone())),
        }
    }
}

pub trait FromSpec: fmt::Debug {
    fn from_spec<S>(spec: S) -> ParsingResult<Self>
    where
        S: AsRef<str>,
        Self: Sized;
}

macro_rules! impl_from_spec_for {
    ($nt:ty, $parser:ty) => {
        impl FromSpec for $nt {
            fn from_spec<S: AsRef<str>>(spec: S) -> ParsingResult<Self> {
                let spec = spec.as_ref();
                let mut errors = Vec::new();

                let result = <$parser>::new().parse(&mut errors, spec).map_err(|err| {
                    err.map_token(|t| format!("{}", t)).map_error(|e| e.to_owned())
                })?;

                Ok(result)
            }
        }
    };
}

impl_from_spec_for!(CesFileBlock, CesFileBlockParser);
impl_from_spec_for!(ImmediateDef, ImmediateDefParser);
impl_from_spec_for!(CesInstance, CesInstanceParser);
impl_from_spec_for!(CapacityBlock, CapBlockParser);
impl_from_spec_for!(MultiplierBlock, MulBlockParser);
impl_from_spec_for!(InhibitorBlock, InhBlockParser);
impl_from_spec_for!(Rex, RexParser);
impl_from_spec_for!(ThinArrowRule, ThinArrowRuleParser);
impl_from_spec_for!(FatArrowRule, FatArrowRuleParser);
impl_from_spec_for!(Polynomial, PolynomialParser);

macro_rules! impl_from_str_for {
    ($nt:ty) => {
        impl FromStr for $nt {
            type Err = ParsingError;

            fn from_str(s: &str) -> ParsingResult<Self> {
                Self::from_spec(s)
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
