use std::{collections::BTreeSet, iter::FromIterator};
use aces::{ContextHandle, NodeId};
use crate::{Node, ToNode, NodeList};

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Warning {
    SumIdempotency(BTreeSet<Node>),
    ProductIdempotency(Node),
}

/// An alphabetically ordered and deduplicated list of monomials,
/// where each monomial is alphabetically ordered and deduplicated
/// list of [`Node`]s.
///
/// The `is_flat` flag indicates whether a `Polynomial` may be
/// interpreted as a [`NodeList`].  The flag is set if the textual
/// form the `Polynomial` originated from was syntactically valid as a
/// node list, or if the `Polynomial` is the result of
/// [`Polynomial::flattened_clone`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Polynomial {
    pub(crate) monomials: BTreeSet<BTreeSet<Node>>,

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

    /// Transform this `Polynomial` into a [`NodeList`]-compatible
    /// form by gathering all [`Node`]s as a single-monomial
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
                        for node in this_mono.intersection(&other_mono) {
                            self.warnings.push(Warning::ProductIdempotency(node.clone()));
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

    pub(crate) fn compile_as_vec(&self, ctx: &ContextHandle) -> Vec<Vec<NodeId>> {
        let mut ctx = ctx.lock().unwrap();

        self.monomials
            .iter()
            .map(|mono| mono.iter().map(|node| ctx.share_node_name(node)).collect())
            .collect()
    }

    pub fn log_warnings(&self) {
        for warning in self.warnings.iter() {
            match warning {
                Warning::SumIdempotency(nodes) => {
                    warn!("Applying sum idempotency of {:?}", nodes);
                }
                Warning::ProductIdempotency(node) => {
                    warn!("Applying product idempotency of {:?}", node);
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

impl From<Node> for Polynomial {
    fn from(node: Node) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(Some(node)))),
            is_flat: true,
            ..Default::default()
        }
    }
}

// FIXME fight with orphan rules, maybe...
impl From<&str> for Polynomial {
    fn from(node: &str) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(Some(node.to_node())))),
            is_flat: true,
            ..Default::default()
        }
    }
}

impl From<Vec<Node>> for Polynomial {
    fn from(mono: Vec<Node>) -> Self {
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
                mono.iter().map(|n| n.to_node()),
            ))),
            is_flat: true,
            ..Default::default()
        }
    }
}

impl From<Vec<Vec<Node>>> for Polynomial {
    fn from(monos: Vec<Vec<Node>>) -> Self {
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
                monos.into_iter().map(|mono| BTreeSet::from_iter(mono.iter().map(|n| n.to_node()))),
            ),
            is_flat: false,
            ..Default::default()
        }
    }
}

impl From<NodeList> for Polynomial {
    fn from(mono: NodeList) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(mono.nodes.iter().cloned()))),
            is_flat: true,
            ..Default::default()
        }
    }
}

impl From<Vec<NodeList>> for Polynomial {
    fn from(monos: Vec<NodeList>) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(
                monos.into_iter().map(|mono| BTreeSet::from_iter(mono.nodes.iter().cloned())),
            ),
            is_flat: false,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ToNode;
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
                        vec!["a".to_node(), "b".to_node(), "d".to_node(), "e".to_node()]
                            .into_iter()
                    ),
                    BTreeSet::from_iter(
                        vec!["a".to_node(), "c".to_node(), "d".to_node(), "e".to_node()]
                            .into_iter()
                    ),
                    BTreeSet::from_iter(vec!["f".to_node(), "g".to_node()].into_iter()),
                ]),
                is_flat: false,
                ..Default::default()
            }
        );
    }
}
