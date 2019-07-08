pub type SymbolID = usize;

#[derive(Clone, Default, Debug)]
pub struct Production {
    lhs: SymbolID,
    rhs: Vec<SymbolID>,
}

#[derive(Default, Debug)]
pub struct Grammar {
    terminals:    Vec<SymbolID>,
    nonterminals: Vec<SymbolID>,
    productions:  Vec<Production>,
    axiom:        SymbolID,
}

impl Grammar {
    pub fn terminals(&self) -> std::slice::Iter<SymbolID> {
        self.terminals.iter()
    }

    pub fn nonterminals(&self) -> std::slice::Iter<SymbolID> {
        self.nonterminals.iter()
    }

    pub fn num_productions(&self) -> usize {
        self.productions.len()
    }

    pub fn productions(&self) -> std::slice::Iter<Production> {
        self.productions.iter()
    }

    pub fn get_axiom(&self) -> SymbolID {
        self.axiom
    }

    pub fn get_lhs(&self, prod_id: usize) -> Option<SymbolID> {
        self.productions.get(prod_id).map(|p| p.lhs)
    }

    pub fn get_rhs(&self, prod_id: usize) -> Option<&[SymbolID]> {
        self.productions.get(prod_id).map(|p| p.rhs.as_slice())
    }

    // FIXME
    pub fn get_rhs_nonterminals(&self, prod_id: usize) -> Option<&[SymbolID]> {
        self.productions.get(prod_id).map(|p| p.rhs.as_slice())
    }
}
