use std::collections::HashMap;
use crate::grammar::{SymbolID, Grammar};

#[derive(Default, Debug)]
pub struct Sentence {
    symbols: Vec<SymbolID>,
}

impl Sentence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, symbol: SymbolID) {
        self.symbols.push(symbol);
    }

    pub fn pop(&mut self, symbol: SymbolID) -> Option<SymbolID> {
        self.symbols.pop()
    }
}

#[derive(Default, Debug)]
pub struct Generator {
    symbol_bounds: HashMap<SymbolID, Option<usize>>, // symbol -> shortest length
    prod_bounds:   Vec<Option<usize>>,               // production index -> shortest length
    shortest_prod: HashMap<SymbolID, Option<usize>>, // nonterminal -> production index
    deriv_bounds:  HashMap<SymbolID, Option<usize>>, // nonterminal -> shortest length
    shortest_prev: HashMap<SymbolID, Option<usize>>, // nonterminal -> production index
}

impl Generator {
    pub fn new() -> Self {
        Self::default()
    }

    fn clear(&mut self) {
        self.symbol_bounds.clear();
        self.prod_bounds.clear();
        self.shortest_prod.clear();
        self.deriv_bounds.clear();
        self.shortest_prev.clear();
    }

    pub fn with_grammar(mut self, grammar: &Grammar) -> Self {
        self.clear();
        
        for t in grammar.terminals() {
            self.symbol_bounds.insert(*t, Some(1));
        }

        for nt in grammar.nonterminals() {
            self.symbol_bounds.insert(*nt, None);
            self.shortest_prod.insert(*nt, None);
        }

        self.prod_bounds.resize(grammar.num_productions(), None);

        loop {
            let mut no_change = true;

            'outer:
            for prod_id in 0..grammar.num_productions() {
                let mut sum = 1;

                for element in grammar.get_rhs(prod_id).unwrap() {
                    if let Some(bound) = self.symbol_bounds[element] {
                        sum += bound;
                    } else {
                        continue 'outer;
                    }
                }

                if self.prod_bounds[prod_id].map_or(true, |v| sum < v) {
                    self.prod_bounds[prod_id] = Some(sum);
                    let lhs = grammar.get_lhs(prod_id).unwrap();

                    if self.symbol_bounds[&lhs].map_or(true, |v| sum < v) {
                        self.symbol_bounds.insert(lhs, Some(sum));
                        self.shortest_prod.insert(lhs, Some(prod_id));
                        no_change = false;
                    }
                }
            }
            if no_change {
                break
            }
        }

        // Compute shortest derivations.

        for nt in grammar.nonterminals() {
            self.deriv_bounds.insert(*nt, None);
            self.shortest_prev.insert(*nt, None);
        }

        let axiom = grammar.get_axiom();

        self.deriv_bounds.insert(axiom, self.symbol_bounds[&axiom]);

        loop {
            let mut no_change = true;

            for prod_id in 0..grammar.num_productions() {
                let lhs = grammar.get_lhs(prod_id).unwrap();

                if let Some(rlen) = self.prod_bounds[prod_id] {
                    if let Some(dlen) = self.deriv_bounds[&lhs] {
                        if let Some(slen) = self.symbol_bounds[&lhs] {

                            let sum = dlen + rlen - slen;

                            for element in grammar.get_rhs_nonterminals(prod_id).unwrap() {
                                if self.deriv_bounds[element].map_or(true, |v| sum < v) {
                                    self.deriv_bounds.insert(*element, Some(sum));
                                    self.shortest_prev.insert(*element, Some(prod_id));
                                    no_change = false;
                                }
                            }
                        }
                    }
                }
            }
            if no_change {
                break
            }
        }
        self
    }

    pub fn generate(&mut self, grammar: &Grammar) -> Sentence {
        let mut result = Sentence::new();
        result
    }
}
