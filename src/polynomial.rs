use std::{collections::BTreeSet, iter::FromIterator};

/// An alphabetically ordered and deduplicated list of monomials,
/// where each monomial is alphabetically ordered and deduplicated
/// list of `Node`s.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Polynomial {
    pub(crate) monomials: BTreeSet<BTreeSet<String>>,

    // FIXME falsify on leading "+" or parens, even if still a mono
    pub(crate) is_flat: bool,
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

    fn multiply_assign(&mut self, factors: &mut [Self]) {
        for factor in factors {
            if !factor.is_flat {
                self.is_flat = false;
            }

            let lhs: Vec<_> = self.monomials.iter().cloned().collect();
            self.monomials.clear();

            for this_mono in lhs.iter() {
                for other_mono in factor.monomials.iter() {
                    let mut mono = this_mono.clone();
                    mono.extend(other_mono.iter().cloned());
                    self.monomials.insert(mono);
                }
            }
        }
    }

    fn add_assign(&mut self, other: &mut Self) {
        self.is_flat = false;
        self.monomials.append(&mut other.monomials);
    }
}

impl Default for Polynomial {
    fn default() -> Self {
        Polynomial { monomials: BTreeSet::default(), is_flat: true }
    }
}

// FIXME impl From<Node>
impl From<String> for Polynomial {
    fn from(node: String) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(Some(node)))),
            is_flat:   true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poly() {
        let spec = "(a (b + c) d e) + f g";
        let poly: Polynomial = spec.parse().unwrap();

        assert_eq!(
            poly,
            Polynomial {
                monomials: BTreeSet::from_iter(vec![
                    BTreeSet::from_iter(
                        vec!["a".to_owned(), "b".to_owned(), "d".to_owned(), "e".to_owned()]
                            .into_iter()
                    ),
                    BTreeSet::from_iter(
                        vec!["a".to_owned(), "c".to_owned(), "d".to_owned(), "e".to_owned()]
                            .into_iter()
                    ),
                    BTreeSet::from_iter(vec!["f".to_owned(), "g".to_owned()].into_iter()),
                ]),
                is_flat:   false,
            }
        );
    }
}
