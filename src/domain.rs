use std::{collections::BTreeSet, convert::TryFrom, iter::FromIterator};
use crate::{Polynomial, AscesisError, AscesisErrorKind};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct DotName(String);

impl From<String> for DotName {
    fn from(id: String) -> Self {
        DotName(id)
    }
}

impl AsRef<str> for DotName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

pub trait ToDotName {
    fn to_dot(&self) -> DotName;
}

impl<S: AsRef<str>> ToDotName for S {
    fn to_dot(&self) -> DotName {
        self.as_ref().to_string().into()
    }
}

/// An alphabetically ordered and deduplicated list of [`DotName`]s.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct DotList {
    pub(crate) dot_names: Vec<DotName>,
}

impl DotList {
    pub fn with_more(mut self, dot_names: Vec<DotName>) -> Self {
        self.dot_names.extend(dot_names.into_iter());
        self.dot_names.sort();
        let len = self.dot_names.partition_dedup().0.len();
        self.dot_names.truncate(len);
        self
    }

    pub(crate) fn add_assign(&mut self, other: &mut Self) {
        self.dot_names.append(&mut other.dot_names);
        self.dot_names.sort();
        let len = self.dot_names.partition_dedup().0.len();
        self.dot_names.truncate(len);
    }
}

impl From<DotName> for DotList {
    fn from(dot: DotName) -> Self {
        DotList { dot_names: vec![dot] }
    }
}

impl<T: ToDotName> From<Vec<T>> for DotList {
    fn from(dot_names: Vec<T>) -> Self {
        let mut dot_names: Vec<DotName> = dot_names.into_iter().map(|n| n.to_dot()).collect();
        dot_names.sort();
        let len = dot_names.partition_dedup().0.len();
        dot_names.truncate(len);

        DotList { dot_names }
    }
}

impl From<BTreeSet<DotName>> for DotList {
    fn from(dot_names: BTreeSet<DotName>) -> Self {
        let dot_names: Vec<DotName> = dot_names.into_iter().map(|n| n.to_dot()).collect();

        DotList { dot_names }
    }
}

impl TryFrom<Polynomial> for DotList {
    type Error = AscesisError;

    fn try_from(poly: Polynomial) -> Result<Self, Self::Error> {
        if poly.is_flat {
            let mut monomials = poly.monomials.into_iter();

            if let Some(monomial) = monomials.next() {
                let dot_names = Vec::from_iter(monomial.into_iter());
                // no need for sorting, unless `monomial` breaks the
                // invariants: 'is-ordered' and 'no-duplicates'...

                if monomials.next().is_none() {
                    Ok(DotList { dot_names })
                } else {
                    Err(AscesisErrorKind::NotADotList.into())
                }
            } else {
                Ok(Default::default())
            }
        } else {
            Err(AscesisErrorKind::NotADotList.into())
        }
    }
}
