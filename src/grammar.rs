use std::ops::Range;

pub type SymbolID = usize;

#[derive(Clone, Default, Debug)]
pub struct Production {
    lhs: SymbolID,
    rhs: Vec<SymbolID>,
    rhs_nonterminals: Vec<SymbolID>,
}

impl Production {
    fn new(lhs: SymbolID) -> Self {
        let mut result = Self::default();
        result.lhs = lhs;
        result
    }

    fn with_rhs(mut self, rhs: &[SymbolID], max_terminal: SymbolID) -> Self {
        self.rhs_nonterminals = rhs.iter().copied().filter(|&id| id >= max_terminal).collect();
        self.rhs = rhs.to_owned();
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

#[derive(Debug)]
pub struct Grammar {
    terminals:    Vec<String>,
    nonterminals: Range<SymbolID>,
    productions:  Vec<Production>,
}

impl Grammar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_terminals(mut self, terminals: &[String]) -> Self {
        self.terminals = terminals.to_vec();
        self.nonterminals = (terminals.len()..terminals.len());
        self.productions.clear();

        self
    }

    // FIXME ignore duplicates
    pub fn add_production(&mut self, lhs: SymbolID, rhs: &[SymbolID]) {
        if rhs.is_empty() {
            self.nonterminals.end = self.nonterminals.end.max(lhs + 1);

            self.productions.push(Production::new(lhs));
        } else {
            self.nonterminals.end = self.nonterminals.end.max(lhs.max(rhs.iter().copied().max().unwrap()) + 1);

            let prod = Production::new(lhs).with_rhs(rhs, self.terminals.len());
            self.productions.push(prod);
        }
    }

    pub fn terminals(&self) -> std::slice::Iter<String> {
        self.terminals.iter()
    }

    pub fn terminal_ids(&self) -> Range<SymbolID> {
        (0..self.terminals.len())
    }

    pub fn nonterminal_ids(&self) -> Range<SymbolID> {
        self.nonterminals.clone()
    }

    pub fn len(&self) -> usize {
        self.productions.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Production> {
        self.productions.iter()
    }

    pub fn get(&self, prod_id: usize) -> Option<&Production> {
        self.productions.get(prod_id)
    }
}

impl Default for Grammar {
    fn default() -> Self {
        Self {
            terminals:    Default::default(),
            nonterminals: (0..0),
            productions:  Default::default(),
        }
    }
}
