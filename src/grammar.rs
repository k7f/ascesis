use std::ops::Range;
use crate::bnf;

/// An integer used to identify a terminal or a nonterminal symbol.
///
/// It easily maps into a symbol table index, see ``.
pub type SymbolID = usize;

/// An integer used to identify a production.
pub type ProductionID = usize;

#[derive(Clone, Default, Debug)]
pub struct Production {
    lhs:              SymbolID,
    rhs:              Vec<SymbolID>,
    rhs_nonterminals: Vec<SymbolID>, // for faster iteration...
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
    /// Symbol table, immutable after grammar is constructed.
    ///
    /// There are two places where it grows: either in
    /// `with_symbols()`, or in `from_bnf()`.  It is split into two
    /// parts, each ordered alphabetically.  Lower part holds
    /// `num_terminals` terminal symbols, upper part holds
    /// `symbols.len()-num_terminals` nonterminal symbols.
    symbols: Vec<String>,

    /// List of productions.
    ///
    /// Productions are ordered by their LHS nonterminal symbol.  The
    /// order inside the group of productions with a common LHS is
    /// arbitrary.
    productions: Vec<Production>,

    /// Number of terminals and the index of the first nonterminal.
    num_terminals: usize,
}

impl Grammar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_bnf(bnf: bnf::Syntax) -> Self {
        let mut result = Self::new();

        // `bnf::Syntax` returns literals in a sorted, deduplicated
        // `Vec`.  Store them in the lower part of the symbol table.
        result.symbols = bnf.get_literals();
        result.num_terminals = result.symbols.len();

        // `bnf::Syntax` returns rules in a sorted, deduplicated
        // slice.  Store their LHS nonterminals in the upper part of
        // the symbol table.
        let nonterminals = bnf.get_rules().iter().map(|rule| rule.get_lhs().to_owned());
        result.symbols.extend(nonterminals);
        result.symbols[result.num_terminals..].sort();

        // Populate the list of productions.
        for (ndx, rule) in bnf.get_rules().iter().enumerate() {
            let lhs = ndx + result.num_terminals;

            let (terminals, nonterminals) = result.symbols.split_at(result.num_terminals);
            let rhs_list = rule.get_rhs_list(terminals, nonterminals);
            for rhs in rhs_list.into_iter() {
                result.push_production(lhs, rhs);
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

        // FIXME deduplication
        self.symbols = terminals.into_iter().collect();
        self.symbols.sort();

        self.num_terminals = self.symbols.len();

        // FIXME deduplication
        self.symbols.extend(nonterminals);
        self.symbols[self.num_terminals..].sort();

        self
    }

    fn push_production(&mut self, lhs: SymbolID, rhs: Vec<SymbolID>) {
        if rhs.is_empty() {
            self.productions.push(Production::new(lhs));
        } else {
            let prod = Production::new(lhs).with_rhs(rhs, self.num_terminals);
            self.productions.push(prod);
        }
    }

    pub fn add_production(&mut self, lhs: SymbolID, rhs: Vec<SymbolID>) {
        // FIXME sorting
        self.push_production(lhs, rhs);
    }

    pub fn terminals(&self) -> std::iter::Take<std::slice::Iter<String>> {
        self.symbols.iter().take(self.num_terminals)
    }

    pub fn terminal_ids(&self) -> Range<SymbolID> {
        (0..self.num_terminals)
    }

    pub fn is_terminal(&self, symbol_id: SymbolID) -> bool {
        symbol_id < self.num_terminals
    }

    pub fn get_terminal(&self, symbol_id: SymbolID) -> Option<&str> {
        if symbol_id < self.num_terminals {
            Some(&self.symbols[symbol_id])
        } else {
            None
        }
    }

    pub fn nonterminal_ids(&self) -> Range<SymbolID> {
        (self.num_terminals..self.symbols.len())
    }

    pub fn id_of_nonterminal<S: AsRef<str>>(&self, name: S) -> Option<SymbolID> {
        let name = name.as_ref();
        self.symbols[self.num_terminals..self.symbols.len()]
            .binary_search_by(|s| s.as_str().cmp(name))
            .ok()
            .map(|id| id + self.num_terminals)
    }

    pub fn is_nonterminal(&self, symbol_id: SymbolID) -> bool {
        symbol_id >= self.num_terminals && symbol_id < self.symbols.len()
    }

    pub fn get_nonterminal(&self, symbol_id: SymbolID) -> Option<&str> {
        if symbol_id >= self.num_terminals && symbol_id < self.symbols.len() {
            Some(&self.symbols[symbol_id])
        } else {
            None
        }
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

    pub fn get(&self, prod_id: ProductionID) -> Option<&Production> {
        self.productions.get(prod_id)
    }
}
