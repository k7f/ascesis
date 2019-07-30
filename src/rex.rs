use std::{convert::TryInto, error::Error};
use crate::{CesInstance, NodeList, BinOp, polynomial::Polynomial};

pub(crate) type RexID = usize;

#[derive(PartialEq, Eq, Default, Debug)]
pub(crate) struct RexTree {
    ids: Vec<RexID>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Rex {
    kinds: Vec<RexKind>,
}

impl Rex {
    pub(crate) fn with_more(self, rexlist: Vec<(Option<BinOp>, Rex)>) -> Self {
        if rexlist.is_empty() {
            return self
        }

        let plusless = rexlist.iter().all(|(op, _)| op.is_none());

        if plusless {
            let mut kinds = vec![RexKind::Product(RexTree::default())];

            let mut ids = vec![1];
            let mut offset = kinds.append_with_offset(self.kinds, 1);

            for (_, rex) in rexlist.into_iter() {
                ids.push(offset);
                offset = kinds.append_with_offset(rex.kinds, offset);
            }

            kinds[0] = RexKind::Product(RexTree { ids });

            Rex { kinds }
        } else {
            // this is used for pruning single-factor products
            let followed_by_product: Vec<bool> =
                rexlist.iter().map(|(op, _)| op.is_none()).collect();
            let mut followed_by_product = followed_by_product.into_iter();

            let mut kinds = vec![RexKind::Sum(RexTree::default())];

            let mut sum_ids = Vec::new();
            let mut product_ids = Vec::new();
            let mut anchor = 1; // index in `kinds` of next addend
            let mut offset = 1; // index in `kinds` of next factor

            if followed_by_product.next().unwrap() {
                kinds.push(RexKind::Product(RexTree::default()));
                offset += 1;
                // `offset` points to first factor of first addend, i.e. to the `self`
                product_ids.push(offset);
            }

            offset = kinds.append_with_offset(self.kinds, offset);
            // `offset` points to expected second factor of first addend or to a second addend

            for (op, rex) in rexlist.into_iter() {
                let is_followed_by_product = followed_by_product.next().unwrap_or(false);

                if let Some(op) = op {
                    if op == BinOp::Add {
                        if !product_ids.is_empty() {
                            if let RexKind::Product(tree) = &mut kinds[anchor] {
                                tree.ids.append(&mut product_ids);
                            } else {
                                panic!()
                            }
                        }

                        sum_ids.push(anchor);
                        anchor = offset;

                        if is_followed_by_product {
                            kinds.push(RexKind::Product(RexTree::default()));
                            offset += 1;
                            product_ids.push(offset);
                        }

                        offset = kinds.append_with_offset(rex.kinds, offset);
                    } else {
                        panic!()
                    }
                } else {
                    product_ids.push(offset);
                    offset = kinds.append_with_offset(rex.kinds, offset);
                }
            }

            if !product_ids.is_empty() {
                kinds[anchor] = RexKind::Product(RexTree { ids: product_ids });
            }
            sum_ids.push(anchor);
            kinds[0] = RexKind::Sum(RexTree { ids: sum_ids });

            Rex { kinds }
        }
    }
}

impl From<ThinArrowRule> for Rex {
    fn from(rule: ThinArrowRule) -> Self {
        Rex { kinds: vec![RexKind::Thin(rule)] }
    }
}

impl From<FatArrowRule> for Rex {
    fn from(rule: FatArrowRule) -> Self {
        Rex { kinds: vec![RexKind::Fat(rule)] }
    }
}

impl From<CesInstance> for Rex {
    fn from(instance: CesInstance) -> Self {
        Rex { kinds: vec![RexKind::Instance(instance)] }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum RexKind {
    Thin(ThinArrowRule),
    Fat(FatArrowRule),
    Instance(CesInstance),
    Product(RexTree),
    Sum(RexTree),
}

trait AppendWithOffset {
    fn append_with_offset(&mut self, source: Self, offset: usize) -> usize;
}

impl AppendWithOffset for Vec<RexKind> {
    fn append_with_offset(&mut self, source: Self, offset: usize) -> usize {
        let result = offset + source.len();

        self.extend(source.into_iter().map(|mut kind| match kind {
            RexKind::Product(ref mut tree) | RexKind::Sum(ref mut tree) => {
                tree.ids.iter_mut().for_each(|id| *id += offset);
                kind
            }
            _ => kind,
        }));

        result
    }
}

#[derive(PartialEq, Eq, Default, Debug)]
pub struct ThinArrowRule {
    nodes:  NodeList,
    cause:  Polynomial,
    effect: Polynomial,
}

impl ThinArrowRule {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn with_nodes(mut self, nodes: Polynomial) -> Result<Self, Box<dyn Error>> {
        self.nodes = nodes.try_into()?;
        Ok(self)
    }

    pub(crate) fn with_cause(mut self, cause: Polynomial) -> Self {
        self.cause = cause;
        self
    }

    pub(crate) fn with_effect(mut self, effect: Polynomial) -> Self {
        self.effect = effect;
        self
    }
}

#[derive(PartialEq, Eq, Default, Debug)]
pub struct FatArrowRule {
    causes:  Vec<Polynomial>,
    effects: Vec<Polynomial>,
}

impl FatArrowRule {
    pub(crate) fn from_parts(head: Polynomial, tail: Vec<(BinOp, Polynomial)>) -> Self {
        assert!(!tail.is_empty(), "Single-polynomial fat rule");

        let mut rule = Self::default();
        let mut prev = head;

        for (op, poly) in tail.into_iter() {
            match op {
                BinOp::FatTx => {
                    rule.causes.push(prev);
                    rule.effects.push(poly.clone());
                }
                BinOp::FatRx => {
                    rule.effects.push(prev);
                    rule.causes.push(poly.clone());
                }
                _ => panic!("Operator not allowed in a fat rule: '{}'.", op),
            }
            prev = poly;
        }
        rule
    }
}

#[cfg(test)]
mod tests {
    use crate::{ToCesName, ToNode};
    use super::*;

    #[test]
    fn test_rex_singles() {
        let spec =
            "{ a => b <= c } { d() + e!(f) g!(h, i) } + { { j -> k -> l } { j -> k } { l <- k } }";
        let rex: Rex = spec.parse().unwrap();

        assert_eq!(
            rex,
            Rex {
                kinds: vec![
                    RexKind::Sum(RexTree { ids: vec![1, 8] }),
                    RexKind::Product(RexTree { ids: vec![2, 3] }),
                    RexKind::Fat(FatArrowRule {
                        causes:  vec![
                            Polynomial::from("a".to_node()),
                            Polynomial::from("c".to_node())
                        ],
                        effects: vec![
                            Polynomial::from("b".to_node()),
                            Polynomial::from("b".to_node())
                        ],
                    }),
                    RexKind::Sum(RexTree { ids: vec![4, 5] }),
                    RexKind::Instance(CesInstance { name: "d".to_ces_name(), args: vec![] }),
                    RexKind::Product(RexTree { ids: vec![6, 7] }),
                    RexKind::Instance(CesInstance {
                        name: "e".to_ces_name(),
                        args: vec!["f".to_string()],
                    }),
                    RexKind::Instance(CesInstance {
                        name: "g".to_ces_name(),
                        args: vec!["h".to_string(), "i".to_string()],
                    }),
                    RexKind::Product(RexTree { ids: vec![9, 10, 11] }),
                    RexKind::Thin(ThinArrowRule {
                        nodes:  NodeList { nodes: vec!["k".to_node()] },
                        cause:  Polynomial::from("j".to_node()),
                        effect: Polynomial::from("l".to_node()),
                    }),
                    RexKind::Thin(ThinArrowRule {
                        nodes:  NodeList { nodes: vec!["j".to_node()] },
                        cause:  Polynomial::default(),
                        effect: Polynomial::from("k".to_node()),
                    }),
                    RexKind::Thin(ThinArrowRule {
                        nodes:  NodeList { nodes: vec!["l".to_node()] },
                        cause:  Polynomial::from("k".to_node()),
                        effect: Polynomial::default(),
                    }),
                ],
            }
        );
    }
}
