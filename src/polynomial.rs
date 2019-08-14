use std::{collections::BTreeSet, iter::FromIterator};
use aces::{ContextHandle, NodeID};
use crate::{Node, ToNode, NodeList};

/// An alphabetically ordered and deduplicated list of monomials,
/// where each monomial is alphabetically ordered and deduplicated
/// list of `Node`s.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Polynomial {
    pub(crate) monomials: BTreeSet<BTreeSet<Node>>,

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

    pub(crate) fn flattened_clone(&self) -> Self {
        if self.is_flat {
            self.clone()
        } else {
            let mut more_monos = self.monomials.iter();
            let mut single_mono = more_monos.next().expect("non-flat empty polynomial").clone();

            for mono in more_monos {
                single_mono.append(&mut mono.clone());
            }

            Polynomial { monomials: BTreeSet::from_iter(Some(single_mono)), is_flat: true }
        }
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

    pub(crate) fn add_assign(&mut self, other: &mut Self) {
        self.is_flat = false;
        self.monomials.append(&mut other.monomials);
    }

    pub(crate) fn into_content(self, ctx: ContextHandle) -> Vec<Vec<NodeID>> {
        let mut ctx = ctx.lock().unwrap();

        self.monomials
            .iter()
            .map(|mono| mono.iter().map(|node| ctx.share_node_name(node)).collect())
            .collect()
    }
}

impl Default for Polynomial {
    fn default() -> Self {
        Polynomial { monomials: BTreeSet::default(), is_flat: true }
    }
}

impl From<Node> for Polynomial {
    fn from(node: Node) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(Some(node)))),
            is_flat:   true,
        }
    }
}

// FIXME fight with orphan rules, maybe...
impl From<&str> for Polynomial {
    fn from(node: &str) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(Some(node.to_node())))),
            is_flat:   true,
        }
    }
}

impl From<Vec<Node>> for Polynomial {
    fn from(mono: Vec<Node>) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(mono.iter().cloned()))),
            is_flat:   true,
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
            is_flat:   true,
        }
    }
}

impl From<Vec<Vec<Node>>> for Polynomial {
    fn from(monos: Vec<Vec<Node>>) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(
                monos.into_iter().map(|mono| BTreeSet::from_iter(mono.iter().cloned())),
            ),
            is_flat:   false,
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
            is_flat:   false,
        }
    }
}

impl From<NodeList> for Polynomial {
    fn from(mono: NodeList) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(Some(BTreeSet::from_iter(mono.nodes.iter().cloned()))),
            is_flat:   true,
        }
    }
}

impl From<Vec<NodeList>> for Polynomial {
    fn from(monos: Vec<NodeList>) -> Self {
        Polynomial {
            monomials: BTreeSet::from_iter(
                monos.into_iter().map(|mono| BTreeSet::from_iter(mono.nodes.iter().cloned())),
            ),
            is_flat:   false,
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
                is_flat:   false,
            }
        );
    }
}
