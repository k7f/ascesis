use std::ops::Range;
use crate::bnf;

pub type SymbolID = usize;

#[derive(Clone, Default, Debug)]
pub struct Production {
    lhs:              SymbolID,
    rhs:              Vec<SymbolID>,
    rhs_nonterminals: Vec<SymbolID>,
}

impl Production {
    fn new(lhs: SymbolID) -> Self {
        let mut result = Self::default();
        result.lhs = lhs;
        result
    }

    fn with_rhs(mut self, rhs: Vec<SymbolID>, max_terminal: SymbolID) -> Self {
        self.rhs_nonterminals = rhs.iter().copied().filter(|&id| id >= max_terminal).collect();
        self.rhs = rhs;
        self
    }

    #[inline]
    pub fn lhs(&self) -> SymbolID {
        self.lhs
    }

    #[inline]
    pub fn rhs(&self) -> &[SymbolID] {
        self.rhs.as_slice()
    }

    #[inline]
    pub fn rhs_nonterminals(&self) -> &[SymbolID] {
        self.rhs_nonterminals.as_slice()
    }
}

#[derive(Default, Debug)]
pub struct Grammar {
    symbols:       Vec<String>,
    productions:   Vec<Production>,
    num_terminals: usize,
}

impl Grammar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bnf(bnf: bnf::Syntax) -> Self {
        let mut result = Self::new();

        result.symbols = bnf.get_literals();
        result.num_terminals = result.symbols.len();

        let nonterminals = bnf.get_rules().iter().map(|rule| rule.get_lhs().to_owned());
        result.symbols.extend(nonterminals);
        result.symbols[result.num_terminals..].sort();

        for (ndx, rule) in bnf.get_rules().iter().enumerate() {
            let lhs = ndx + result.num_terminals;

            let (terminals, nonterminals) = result.symbols.split_at(result.num_terminals);
            let rhs_list = rule.get_rhs_list(terminals, nonterminals);
            for rhs in rhs_list.into_iter() {
                result.add_production(lhs, rhs);
            }
        }

        result
    }

    pub fn of_cesar() -> Self {
        Self::from_bnf(bnf::Syntax::of_cesar())
    }

    pub fn with_symbols<I, J>(mut self, terminals: I, nonterminals: J) -> Self
    where
        I: IntoIterator<Item = String>,
        J: IntoIterator<Item = String>,
    {
        self.productions.clear();

        self.symbols = terminals.into_iter().collect();
        self.symbols.sort();

        self.num_terminals = self.symbols.len();

        self.symbols.extend(nonterminals);
        self.symbols[self.num_terminals..].sort();

        self
    }

    pub fn add_production(&mut self, lhs: SymbolID, rhs: Vec<SymbolID>) {
        if rhs.is_empty() {
            self.productions.push(Production::new(lhs));
        } else {
            let prod = Production::new(lhs).with_rhs(rhs, self.num_terminals);
            self.productions.push(prod);
        }
    }

    pub fn terminals(&self) -> std::iter::Take<std::slice::Iter<String>> {
        self.symbols.iter().take(self.num_terminals)
    }

    pub fn terminal_ids(&self) -> Range<SymbolID> {
        (0..self.num_terminals)
    }

    pub fn nonterminal_ids(&self) -> Range<SymbolID> {
        (self.num_terminals..self.symbols.len())
    }

    pub fn len(&self) -> usize {
        self.productions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.productions.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<Production> {
        self.productions.iter()
    }

    pub fn get(&self, prod_id: usize) -> Option<&Production> {
        self.productions.get(prod_id)
    }
}
