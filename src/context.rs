use std::{collections::BTreeMap, convert::TryInto, cmp, fmt, error::Error};
use aces::{ContextHandle, Compilable, Polarity, Capacity, Weight, sat};
use crate::{Polynomial, DotName, DotList, Literal, AscesisError, AscesisErrorKind};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PropSelector {
    AnonymousBlock,
    Vis,
    SAT,
    Invalid(String),
}

impl Default for PropSelector {
    #[inline]
    fn default() -> Self {
        PropSelector::AnonymousBlock
    }
}

impl fmt::Display for PropSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PropSelector::*;

        match self {
            AnonymousBlock => write!(f, "anonymous block"),
            Vis => write!(f, "Vis"),
            SAT => write!(f, "SAT"),
            Invalid(ref name) => write!(f, "{}", name),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PropValue {
    Literal(Literal),
    Identifier(String),
    SizeList(Vec<Literal>),
    IdentifierList(Vec<String>),
    DotList(DotList),
    Array(Vec<PropValue>),
    Block(PropBlock),
}

impl PropValue {
    pub(crate) fn into_array_with_more(self, more: Vec<Self>) -> Self {
        let mut values = match self {
            PropValue::Array(v) => v,
            _ => vec![self],
        };

        values.extend(more.into_iter());

        PropValue::Array(values)
    }

    pub(crate) fn new_name(lit: Literal) -> Result<Self, AscesisError> {
        if matches!(lit, Literal::Name(_)) {
            Ok(PropValue::Literal(lit))
        } else {
            Err(AscesisErrorKind::InvalidPropValueType("Name".into()).into())
        }
    }

    pub(crate) fn new_size_list(lits: Vec<Literal>) -> Result<Self, AscesisError> {
        if lits.iter().all(|lit| matches!(lit, Literal::Size(_))) {
            Ok(PropValue::SizeList(lits))
        } else {
            Err(AscesisErrorKind::InvalidPropValueType("Size".into()).into())
        }
    }

    pub(crate) fn new_dot_list(names: Vec<String>) -> Result<Self, AscesisError> {
        Ok(PropValue::DotList(names.into()))
    }
}

impl From<PropBlock> for PropValue {
    fn from(block: PropBlock) -> Self {
        PropValue::Block(block)
    }
}

impl From<Vec<PropValue>> for PropValue {
    fn from(vals: Vec<PropValue>) -> Self {
        PropValue::Array(vals)
    }
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct PropBlock {
    selector: PropSelector,
    fields:   BTreeMap<String, PropValue>,
}

impl PropBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn with_prop(mut self, key: String, value: PropValue) -> Self {
        self.fields.insert(key, value);

        self
    }

    pub(crate) fn with_selector(mut self, selector: String) -> Self {
        match selector.as_str() {
            "vis" => self.selector = PropSelector::Vis,
            "sat" => self.selector = PropSelector::SAT,
            _ => self.selector = PropSelector::Invalid(selector),
        }

        self
    }

    pub fn get_selector(&self) -> Result<PropSelector, AscesisError> {
        if let PropSelector::Invalid(ref selector) = self.selector {
            Err(AscesisErrorKind::InvalidPropSelector(selector.to_owned()).into())
        } else {
            Ok(self.selector.clone())
        }
    }

    pub fn verify_selector(&self, expected: PropSelector) -> Result<(), AscesisError> {
        let actual = self.get_selector()?;

        if actual == expected {
            Ok(())
        } else {
            Err(AscesisErrorKind::BlockSelectorMismatch(expected, actual).into())
        }
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.fields.append(&mut block.fields);
        }
        self
    }

    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<&PropValue> {
        let key = key.as_ref();

        self.fields.get(key)
    }

    pub fn get_size<S: AsRef<str>>(&self, key: S) -> Option<u64> {
        let key = key.as_ref();

        self.fields.get(key).and_then(|value| {
            if let PropValue::Literal(Literal::Size(size)) = value {
                Some(*size)
            } else {
                None
            }
        })
    }

    pub fn get_name<S: AsRef<str>>(&self, key: S) -> Option<&str> {
        let key = key.as_ref();

        self.fields.get(key).and_then(|value| {
            if let PropValue::Literal(Literal::Name(name)) = value {
                Some(name.as_str())
            } else {
                None
            }
        })
    }

    pub fn get_identifier<S: AsRef<str>>(&self, key: S) -> Option<&str> {
        let key = key.as_ref();

        self.fields.get(key).and_then(|value| {
            if let PropValue::Identifier(identifier) = value {
                Some(identifier.as_str())
            } else {
                None
            }
        })
    }

    pub fn get_name_or_identifier<S: AsRef<str>>(
        &self,
        key: S,
    ) -> Result<Option<&str>, AscesisError> {
        let key = key.as_ref();

        if let Some(value) = self.fields.get(key) {
            match value {
                PropValue::Literal(Literal::Name(name)) => return Ok(Some(name.as_str())),
                PropValue::Identifier(identifier) => return Ok(Some(identifier.as_str())),
                PropValue::DotList(dot_list) => {
                    if dot_list.dot_names.len() == 1 {
                        return Ok(dot_list.dot_names.first().map(|n| n.as_ref()))
                    }
                }
                PropValue::IdentifierList(ids) => {
                    if ids.len() == 1 {
                        return Ok(ids.first().map(|i| i.as_str()))
                    }
                }
                _ => {}
            };

            Err(AscesisErrorKind::InvalidPropType(self.selector.clone(), "key".to_owned()).into())
        } else {
            Ok(None)
        }
    }

    pub fn get_nested_size<I, S>(&self, subblock_keys: I, value_key: S) -> Option<u64>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        S: AsRef<str>,
    {
        let mut block_keys = subblock_keys.into_iter();

        if let Some(block_key) = block_keys.next() {
            let block_key = block_key.as_ref();

            self.fields.get(block_key).and_then(|value| {
                if let PropValue::Block(block) = value {
                    block.get_nested_size(block_keys, value_key)
                } else {
                    None
                }
            })
        } else {
            self.get_size(value_key)
        }
    }

    pub fn get_nested_name<I, S>(&self, subblock_keys: I, value_key: S) -> Option<&str>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        S: AsRef<str>,
    {
        let mut block_keys = subblock_keys.into_iter();

        if let Some(block_key) = block_keys.next() {
            let block_key = block_key.as_ref();

            self.fields.get(block_key).and_then(|value| {
                if let PropValue::Block(block) = value {
                    block.get_nested_name(block_keys, value_key)
                } else {
                    None
                }
            })
        } else {
            self.get_name(value_key)
        }
    }

    pub fn get_nested_identifier<I, S>(&self, subblock_keys: I, value_key: S) -> Option<&str>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
        S: AsRef<str>,
    {
        let mut block_keys = subblock_keys.into_iter();

        if let Some(block_key) = block_keys.next() {
            let block_key = block_key.as_ref();

            self.fields.get(block_key).and_then(|value| {
                if let PropValue::Block(block) = value {
                    block.get_nested_identifier(block_keys, value_key)
                } else {
                    None
                }
            })
        } else {
            self.get_identifier(value_key)
        }
    }

    pub fn get_sat_encoding(&self) -> Result<Option<sat::Encoding>, AscesisError> {
        self.verify_selector(PropSelector::SAT)?;

        if let Some(encoding) = self.get_name_or_identifier("encoding")? {
            match encoding {
                "port-link" => Ok(Some(sat::Encoding::PortLink)),
                "fork-join" => Ok(Some(sat::Encoding::ForkJoin)),
                _ => Err(AscesisErrorKind::InvalidPropValue(
                    PropSelector::SAT,
                    "encoding".to_owned(),
                    encoding.to_owned(),
                )
                .into()),
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_sat_search(&self) -> Result<Option<sat::Search>, AscesisError> {
        self.verify_selector(PropSelector::SAT)?;

        if let Some(search) = self.get_name_or_identifier("search")? {
            match search {
                "min" => Ok(Some(sat::Search::MinSolutions)),
                "all" => Ok(Some(sat::Search::AllSolutions)),
                _ => Err(AscesisErrorKind::InvalidPropValue(
                    PropSelector::SAT,
                    "search".to_owned(),
                    search.to_owned(),
                )
                .into()),
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_vis_title(&self) -> Result<Option<&str>, AscesisError> {
        self.verify_selector(PropSelector::Vis)?;

        Ok(self.get_name_or_identifier("title")?)
    }

    pub fn get_vis_labels(&self) -> Result<Option<&BTreeMap<String, PropValue>>, AscesisError> {
        self.verify_selector(PropSelector::Vis)?;

        if let Some(value) = self.fields.get("labels") {
            if let PropValue::Block(block) = value {
                Ok(Some(&block.fields))
            } else {
                Err(AscesisErrorKind::InvalidPropType(self.selector.clone(), "labels".to_owned())
                    .into())
            }
        } else {
            Ok(None)
        }
    }
}

impl Compilable for PropBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        match self.get_selector()? {
            PropSelector::Vis => {
                if let Some(title) = self.get_vis_title()? {
                    ctx.lock().unwrap().set_title(title);
                }

                if let Some(labels) = self.get_vis_labels()? {
                    for (dot_name, dot_label) in labels {
                        match dot_label {
                            PropValue::Literal(Literal::Name(ref label))
                            | PropValue::Identifier(ref label) => {
                                let mut ctx = ctx.lock().unwrap();
                                let dot_id = ctx.share_dot_name(dot_name);

                                ctx.set_label(dot_id, label);
                            }
                            _ => {
                                return Err(AscesisError::from(AscesisErrorKind::InvalidPropType(
                                    PropSelector::Vis,
                                    "labels".to_owned(),
                                ))
                                .into())
                            }
                        }
                    }
                }
            }

            PropSelector::SAT => {
                if let Some(encoding) = self.get_sat_encoding()? {
                    info!("Using encoding '{:?}'", encoding);
                    ctx.lock().unwrap().set_encoding(encoding);
                }

                if let Some(search) = self.get_sat_search()? {
                    info!("Using '{:?}' search", search);
                    ctx.lock().unwrap().set_search(search);
                }
            }

            _ => unreachable!(),
        }

        Ok(true)
    }
}

/// A map from dots to their capacities.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct CapacitiesBlock {
    capacities: BTreeMap<DotName, Capacity>,
}

impl CapacitiesBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn with_dot_names(
        mut self,
        size: Literal,
        dot_names: Polynomial,
    ) -> Result<Self, AscesisError> {
        let capacity = match size {
            Literal::Size(sz) => Capacity::finite(sz)
                .ok_or_else(|| AscesisError::from(AscesisErrorKind::SizeLiteralOverflow))?,
            Literal::Omega => Capacity::omega(),
            _ => return Err(AscesisError::from(AscesisErrorKind::ExpectedSizeLiteral)),
        };
        let dot_list: DotList = dot_names.try_into()?;

        for dot_name in dot_list.dot_names.into_iter() {
            self.capacities.insert(dot_name, capacity);
        }

        Ok(self)
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.capacities.append(&mut block.capacities);
        }
        self
    }
}

impl Compilable for CapacitiesBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for (dot_name, cap) in self.capacities.iter() {
            ctx.set_capacity_by_name(dot_name.as_ref(), *cap);
        }

        Ok(true)
    }
}

/// A vector of unbounded capacity dots.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct UnboundedBlock {
    dot_names: Vec<DotName>,
}

impl UnboundedBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn from_dot_names(dot_names: Polynomial) -> Result<Self, AscesisError> {
        let dot_list: DotList = dot_names.try_into()?;

        Ok(UnboundedBlock { dot_names: dot_list.dot_names })
    }
}

impl Compilable for UnboundedBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for dot_name in self.dot_names.iter() {
            ctx.set_capacity_by_name(dot_name.as_ref(), Capacity::omega());
        }

        Ok(true)
    }
}

/// An alphabetically ordered and deduplicated list of transfer
/// multiplicities.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct WeightsBlock {
    xfer_multiplicities: Vec<XferMultiplicity>,
}

impl WeightsBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn new_join_weights(
        size: Literal,
        post_dots: Polynomial,
        pre_arms: Polynomial,
    ) -> Result<Self, AscesisError> {
        let weight = size.try_into()?;
        let post_dots: DotList = post_dots.try_into()?;
        let pre_arms: DotList = pre_arms.try_into()?;

        let xfer_multiplicities = post_dots
            .dot_names
            .into_iter()
            .map(|tip_name| {
                XferMultiplicity::Rx(RxWeight { weight, tip_name, pre_arms: pre_arms.clone() })
            })
            .collect();
        // No need to sort: `post_dots` are already ordered and deduplicated.

        Ok(WeightsBlock { xfer_multiplicities })
    }

    pub fn new_fork_weights(
        size: Literal,
        pre_dots: Polynomial,
        post_arms: Polynomial,
    ) -> Result<Self, AscesisError> {
        let weight = size.try_into()?;
        let pre_dots: DotList = pre_dots.try_into()?;
        let post_arms: DotList = post_arms.try_into()?;

        let xfer_multiplicities = pre_dots
            .dot_names
            .into_iter()
            .map(|tip_name| {
                XferMultiplicity::Tx(TxWeight { weight, tip_name, post_arms: post_arms.clone() })
            })
            .collect();
        // No need to sort: `pre_dots` are already ordered and deduplicated.

        Ok(WeightsBlock { xfer_multiplicities })
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.xfer_multiplicities.append(&mut block.xfer_multiplicities);
        }

        self.xfer_multiplicities.sort();
        let len = self.xfer_multiplicities.partition_dedup().0.len();
        self.xfer_multiplicities.truncate(len);

        self
    }
}

impl Compilable for WeightsBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for weight in self.xfer_multiplicities.iter() {
            match weight {
                XferMultiplicity::Rx(rx) => {
                    let pre_names = rx.pre_arms.dot_names.iter().map(|n| n.as_ref());

                    ctx.set_wedge_weight_by_names(
                        Polarity::Rx,
                        rx.tip_name.as_ref(),
                        pre_names,
                        rx.weight,
                    );
                }
                XferMultiplicity::Tx(tx) => {
                    let post_names = tx.post_arms.dot_names.iter().map(|n| n.as_ref());

                    ctx.set_wedge_weight_by_names(
                        Polarity::Tx,
                        tx.tip_name.as_ref(),
                        post_names,
                        tx.weight,
                    );
                }
            }
        }

        Ok(true)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum XferMultiplicity {
    Rx(RxWeight),
    Tx(TxWeight),
}

impl cmp::Ord for XferMultiplicity {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self {
            Self::Rx(s) => match other {
                Self::Rx(o) => s.cmp(o),
                Self::Tx(_) => cmp::Ordering::Less,
            },
            Self::Tx(s) => match other {
                Self::Rx(_) => cmp::Ordering::Greater,
                Self::Tx(o) => s.cmp(o),
            },
        }
    }
}

impl cmp::PartialOrd for XferMultiplicity {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct RxWeight {
    weight:   Weight,
    tip_name: DotName,
    pre_arms: DotList,
}

impl cmp::Ord for RxWeight {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.tip_name.cmp(&other.tip_name) {
            cmp::Ordering::Equal => match self.pre_arms.cmp(&other.pre_arms) {
                cmp::Ordering::Equal => self.weight.cmp(&other.weight),
                result => result,
            },
            result => result,
        }
    }
}

impl cmp::PartialOrd for RxWeight {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct TxWeight {
    weight:    Weight,
    tip_name:  DotName,
    post_arms: DotList,
}

impl cmp::Ord for TxWeight {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.tip_name.cmp(&other.tip_name) {
            cmp::Ordering::Equal => match self.post_arms.cmp(&other.post_arms) {
                cmp::Ordering::Equal => self.weight.cmp(&other.weight),
                result => result,
            },
            result => result,
        }
    }
}

impl cmp::PartialOrd for TxWeight {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// An alphabetically ordered and deduplicated list of `Inhibitor`s.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct InhibitorsBlock {
    inhibitors: Vec<Inhibitor>,
}

impl InhibitorsBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn new_causes(post_dots: Polynomial, pre_poly: Polynomial) -> Result<Self, AscesisError> {
        let post_dots: DotList = post_dots.try_into()?;
        let mut inhibitors = Vec::new();

        // `post_dots` are already ordered and deduplicated
        for post_dot in post_dots.dot_names {
            // monomials are already ordered and deduplicated
            for mono in pre_poly.monomials.iter() {
                let post_tip = post_dot.clone();
                let pre_arms = mono.clone().into();

                inhibitors.push(Inhibitor::Rx(RxInhibitor { post_tip, pre_arms }));
            }
        }

        Ok(InhibitorsBlock { inhibitors })
    }

    pub fn new_effects(pre_dots: Polynomial, post_poly: Polynomial) -> Result<Self, AscesisError> {
        let pre_dots: DotList = pre_dots.try_into()?;
        let mut inhibitors = Vec::new();

        // `pre_dots` are already ordered and deduplicated
        for pre_dot in pre_dots.dot_names {
            // monomials are already ordered and deduplicated
            for mono in post_poly.monomials.iter() {
                let pre_tip = pre_dot.clone();
                let post_arms = mono.clone().into();

                inhibitors.push(Inhibitor::Tx(TxInhibitor { pre_tip, post_arms }));
            }
        }

        Ok(InhibitorsBlock { inhibitors })
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            self.inhibitors.append(&mut block.inhibitors);
        }

        self.inhibitors.sort();
        let len = self.inhibitors.partition_dedup().0.len();
        self.inhibitors.truncate(len);

        self
    }
}

impl Compilable for InhibitorsBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for inhibitor in self.inhibitors.iter() {
            match inhibitor {
                Inhibitor::Rx(rx) => {
                    let arm_names = rx.pre_arms.dot_names.iter().map(|n| n.as_ref());

                    ctx.set_wedge_inhibitor_by_names(Polarity::Rx, rx.post_tip.as_ref(), arm_names);
                }
                Inhibitor::Tx(tx) => {
                    let arm_names = tx.post_arms.dot_names.iter().map(|n| n.as_ref());

                    ctx.set_wedge_inhibitor_by_names(Polarity::Tx, tx.pre_tip.as_ref(), arm_names);
                }
            }
        }

        Ok(true)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Inhibitor {
    Rx(RxInhibitor),
    Tx(TxInhibitor),
}

impl cmp::Ord for Inhibitor {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self {
            Self::Rx(s) => match other {
                Self::Rx(o) => s.cmp(o),
                Self::Tx(_) => cmp::Ordering::Less,
            },
            Self::Tx(s) => match other {
                Self::Rx(_) => cmp::Ordering::Greater,
                Self::Tx(o) => s.cmp(o),
            },
        }
    }
}

impl cmp::PartialOrd for Inhibitor {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RxInhibitor {
    post_tip: DotName,
    pre_arms: DotList,
}

impl cmp::Ord for RxInhibitor {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.post_tip.cmp(&other.post_tip) {
            cmp::Ordering::Equal => self.pre_arms.cmp(&other.pre_arms),
            result => result,
        }
    }
}

impl cmp::PartialOrd for RxInhibitor {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TxInhibitor {
    pre_tip:   DotName,
    post_arms: DotList,
}

impl cmp::Ord for TxInhibitor {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.pre_tip.cmp(&other.pre_tip) {
            cmp::Ordering::Equal => self.post_arms.cmp(&other.post_arms),
            result => result,
        }
    }
}

impl cmp::PartialOrd for TxInhibitor {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// An alphabetically ordered and deduplicated list of `Weightless` splits.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct WeightlessBlock {
    pub(crate) polarity: Option<Polarity>,
    pub(crate) splits:   Vec<Weightless>,
}

impl WeightlessBlock {
    #[inline]
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn new_causes(post_dots: Polynomial, pre_poly: Polynomial) -> Result<Self, AscesisError> {
        let polarity = Some(Polarity::Rx);
        let post_dots: DotList = post_dots.try_into()?;
        let mut splits = Vec::new();

        // `post_dots` are already ordered and deduplicated
        for post_dot in post_dots.dot_names {
            // monomials are already ordered and deduplicated
            for mono in pre_poly.monomials.iter() {
                let post_tip = post_dot.clone();
                let pre_arms = mono.clone().into();

                splits.push(Weightless::Drop(RxWeightless { post_tip, pre_arms }));
            }
        }

        Ok(WeightlessBlock { polarity, splits })
    }

    pub fn new_effects(pre_dots: Polynomial, post_poly: Polynomial) -> Result<Self, AscesisError> {
        let polarity = Some(Polarity::Tx);
        let pre_dots: DotList = pre_dots.try_into()?;
        let mut splits = Vec::new();

        // `pre_dots` are already ordered and deduplicated
        for pre_dot in pre_dots.dot_names {
            // monomials are already ordered and deduplicated
            for mono in post_poly.monomials.iter() {
                let pre_tip = pre_dot.clone();
                let post_arms = mono.clone().into();

                splits.push(Weightless::Activate(TxWeightless { pre_tip, post_arms }));
            }
        }

        Ok(WeightlessBlock { polarity, splits })
    }

    pub(crate) fn with_more(mut self, more: Vec<Self>) -> Self {
        for mut block in more {
            if self.polarity.is_some() && block.polarity != self.polarity {
                self.polarity = None;
            }

            self.splits.append(&mut block.splits);
        }

        self.splits.sort();
        let len = self.splits.partition_dedup().0.len();
        self.splits.truncate(len);

        self
    }

    #[inline]
    pub fn get_polarity(&self) -> Option<Polarity> {
        self.polarity
    }
}

impl From<WeightlessBlock> for WeightsBlock {
    fn from(block: WeightlessBlock) -> Self {
        let mut more_weights = Vec::new();

        for split in block.splits {
            // FIXME unwraps
            match split {
                Weightless::Activate(activate) => {
                    more_weights.push(
                        WeightsBlock::new_fork_weights(
                            Literal::Size(0),
                            activate.pre_tip.into(),
                            activate.post_arms.into(),
                        )
                        .unwrap(),
                    );
                }
                Weightless::Drop(drop) => {
                    more_weights.push(
                        WeightsBlock::new_join_weights(
                            Literal::Size(0),
                            drop.post_tip.into(),
                            drop.pre_arms.into(),
                        )
                        .unwrap(),
                    );
                }
            }
        }

        WeightsBlock::new().with_more(more_weights)
    }
}

impl Compilable for WeightlessBlock {
    fn compile(&self, ctx: &ContextHandle) -> Result<bool, Box<dyn Error>> {
        let mut ctx = ctx.lock().unwrap();

        for activator in self.splits.iter() {
            match activator {
                Weightless::Activate(tx) => {
                    let arm_names = tx.post_arms.dot_names.iter().map(|n| n.as_ref());

                    ctx.set_wedge_activator_by_names(Polarity::Tx, tx.pre_tip.as_ref(), arm_names);
                }
                Weightless::Drop(rx) => {
                    let arm_names = rx.pre_arms.dot_names.iter().map(|n| n.as_ref());

                    ctx.set_wedge_activator_by_names(Polarity::Rx, rx.post_tip.as_ref(), arm_names);
                }
            }
        }

        Ok(true)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Weightless {
    Activate(TxWeightless),
    Drop(RxWeightless),
}

impl cmp::Ord for Weightless {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self {
            Self::Activate(s) => match other {
                Self::Activate(o) => s.cmp(o),
                Self::Drop(_) => cmp::Ordering::Greater,
            },
            Self::Drop(s) => match other {
                Self::Activate(_) => cmp::Ordering::Less,
                Self::Drop(o) => s.cmp(o),
            },
        }
    }
}

impl cmp::PartialOrd for Weightless {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TxWeightless {
    pre_tip:   DotName,
    post_arms: DotList,
}

impl cmp::Ord for TxWeightless {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.pre_tip.cmp(&other.pre_tip) {
            cmp::Ordering::Equal => self.post_arms.cmp(&other.post_arms),
            result => result,
        }
    }
}

impl cmp::PartialOrd for TxWeightless {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RxWeightless {
    post_tip: DotName,
    pre_arms: DotList,
}

impl cmp::Ord for RxWeightless {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.post_tip.cmp(&other.post_tip) {
            cmp::Ordering::Equal => self.pre_arms.cmp(&other.pre_arms),
            result => result,
        }
    }
}

impl cmp::PartialOrd for RxWeightless {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
