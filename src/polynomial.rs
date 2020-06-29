use std::{collections::BTreeSet, iter::FromIterator};
use aces::{ContextHandle, DotId};
use crate::{DotName, ToDotName, DotList};

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Warning {
    SumIdempotency(BTreeSet<DotName>),
    ProductIdempotency(DotName),
}

/// An alphabetically ordered and deduplicated list of monomials,
/// where each monomial is alphabetically ordered and deduplicated
/// list of [`DotName`]s.
///
/// The `is_flat` flag indicates whether a `Polynomial` may be
/// interpreted as a [`DotList`].  The flag is set if the textual form
/// the `Polynomial` originated from was syntactically valid as a dot
/// list, or if the `Polynomial` is the result of
/// [`Polynomial::flattened_clone`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Polynomial {
    pub(crate) monomials: BTreeSet<BTreeSet<DotName>>,

    // FIXME falsify on leading "+" or parens, even if still a single mono
    pub(crate) is_flat:  bool,
    pub(crate) warnings: Vec<Warning>,
}

impl Polynomial {
    /// Returns `self` multiplied by the product of `factors`.
    pub(crate) fn with_product_multiplied(mut self, mut factors: Vec<Self>) -> Self {
        self.multiply_assign(&mut factors);
        self
    }

    /// Returns `self` added to the product of `factors`.
    pub(crate) fn with_product_added(mut self, mut factors: Vec<Self>) -> Self {
        if let Some((head, tail)) = factors.split_first_mut() {
            head.multiply_assign(tail);
            self.add_assign(head);
        }
        self
    }

    /// Transform this `Polynomial` into a [`DotList`]-compatible form
    /// by gathering all [`DotName`]s as a single-monomial
    /// `Polynomial` with the `is_flat` flag set.
    pub(crate) fn flattened_clone(&self) -> Self {
        if self.is_flat {
            self.clone()
        } else {
            let warnings = self.warnings.clone();
            let mut more_monos = self.monomials.iter();
            let mut single_mono = more_monos.next().expect("non-flat empty polynomial").clone();

            for mono in more_monos {
                single_mono.append(&mut mono.clone());
            }

            Polynomial {
                monomials: BTreeSet::from_iter(Some(single_mono)),
                is_flat: true,
                warnings,
            }
        }
    }

    pub(crate) fn multiply_assign(&mut self, factors: &mut [Self]) {
        for factor in factors {
            if !factor.is_flat {
                self.is_flat = false;
            }

            let lhs: Vec<_> = self.monomials.iter().cloned().collect();
            self.monomials.clear();

            for this_mono in lhs.iter() {
                for other_mono in factor.monomials.iter() {
                    if !this_mono.is_disjoint(other_mono) {
                        for dot in this_mono.intersection(&other_mono) {
                            self.warnings.push(Warning::ProductIdempotency(dot.clone()));
                        }
                    }

                    let mut mono = this_mono.clone();
                    mono.extend(other_mono.iter().cloned());
                    self.monomials.insert(mono);
                }
            }
        }
        self.log_warnings();
    }

    pub(crate) fn add_assign(&mut self, other: &mut Self) {
        self.is_flat = false;

        if !self.monomials.is_disjoint(&other.monomials) {
            for mono in self.monomials.intersection(&other.monomials) {
                self.warnings.push(Warning::SumIdempotency(mono.clone()));
            }
        }

        self.monomials.append(&mut other.monomials);
        self.log_warnings();
    }

    pub(crate) fn compile_as_vec(&self, ctx: &ContextHandle) -> Vec<Vec<DotId>> {
        let mut ctx = ctx.lock().unwrap();

        self.monomials
            .iter()
            .map(|mono| mono.iter().map(|dot| ctx.share_dot_name(dot)).collect())
            .collect()
    }

    pub fn log_warnings(&self) {
        for warning in self.warnings.iter() {
            match warning {
                Warning::SumIdempotency(dots) => {
                    warn!("Applying sum idempotency of {:?}", dots);
                }
                Warning::ProductIdempotency(dot) => {
                    warn!("Applying product idempotency of {:?}", dot);
                }
            }
        }
    }
}

impl Default for Polynomial {
    fn default() -> Self {
        Polynomial { monomials: BTreeSet::default(), is_flat: true, warnings: Vec::new() }
    }
}

impl From<DotName> for Polynomial {
    fn from(dot: DotName) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(Some(dot)))),
            is_flat: true,
            ..Default::default()
        }
    }
}

// FIXME fight with orphan rules, maybe...
impl From<&str> for Polynomial {
    fn from(dot: &str) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(Some(dot.to_dot())))),
            is_flat: true,
            ..Default::default()
        }
    }
}

impl From<Vec<DotName>> for Polynomial {
    fn from(mono: Vec<DotName>) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(mono.iter().cloned()))),
            is_flat: true,
            ..Default::default()
        }
    }
}

// FIXME fight with orphan rules, maybe...
impl From<Vec<&str>> for Polynomial {
    fn from(mono: Vec<&str>) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(
                mono.iter().map(|n| n.to_dot()),
            ))),
            is_flat: true,
            ..Default::default()
        }
    }
}

impl From<Vec<Vec<DotName>>> for Polynomial {
    fn from(monos: Vec<Vec<DotName>>) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(
                monos.into_iter().map(|mono| BTreeSet::from_iter(mono.iter().cloned())),
            ),
            is_flat: false,
            ..Default::default()
        }
    }
}

// FIXME fight with orphan rules, maybe...
impl From<Vec<Vec<&str>>> for Polynomial {
    fn from(monos: Vec<Vec<&str>>) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(
                monos.into_iter().map(|mono| BTreeSet::from_iter(mono.iter().map(|n| n.to_dot()))),
            ),
            is_flat: false,
            ..Default::default()
        }
    }
}

impl From<DotList> for Polynomial {
    fn from(mono: DotList) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(
                mono.dot_names.iter().cloned(),
            ))),
            is_flat: true,
            ..Default::default()
        }
    }
}

impl From<Vec<DotList>> for Polynomial {
    fn from(monos: Vec<DotList>) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(
                monos.into_iter().map(|mono| BTreeSet::from_iter(mono.dot_names.iter().cloned())),
            ),
            is_flat: false,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ToDotName;
    use super::*;

    #[test]
    fn test_poly() {
        let phrase = "(a (b + c) d e) + f g";
        let poly: Polynomial = phrase.parse().unwrap();

        assert_eq!(
            poly,
            Polynomial {
                monomials: BTreeSet::from_iter(vec![
                    BTreeSet::from_iter(
                        vec!["a".to_dot(), "b".to_dot(), "d".to_dot(), "e".to_dot()].into_iter()
                    ),
                    BTreeSet::from_iter(
                        vec!["a".to_dot(), "c".to_dot(), "d".to_dot(), "e".to_dot()].into_iter()
                    ),
                    BTreeSet::from_iter(vec!["f".to_dot(), "g".to_dot()].into_iter()),
                ]),
                is_flat: false,
                ..Default::default()
            }
        );
    }
}
