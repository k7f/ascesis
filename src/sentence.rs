use std::collections::HashMap;
use crate::grammar::{Grammar, SymbolID, ProductionID};

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

    pub fn pop(&mut self) -> Option<SymbolID> {
        self.symbols.pop()
    }
}

#[derive(PartialEq, Eq, Debug)]
enum ProductionUsed {
    Ready,
    Unsure,
    Finished,
    ID(ProductionID),
}

impl Default for ProductionUsed {
    fn default() -> Self {
        ProductionUsed::Ready
    }
}

#[derive(Default, Debug)]
pub struct Generator {
    // axiom-independent derivation data
    symbol_min:  HashMap<SymbolID, Option<usize>>, // symbol -> shortest length
    prod_min:    Vec<Option<usize>>,               // production index -> shortest length
    best_prod:   HashMap<SymbolID, Option<usize>>, // nonterminal -> production index

    // axiom-specific derivation data
    axiom:       Option<SymbolID>,
    min_through: HashMap<SymbolID, Option<usize>>, // nonterminal -> shortest length
    best_parent: HashMap<SymbolID, Option<usize>>, // nonterminal -> production index

    // state of emission
    which_prod:  HashMap<SymbolID, ProductionUsed>, // nonterminal -> production used
    on_stack:    HashMap<SymbolID, usize>,          // nonterminal -> #occurences on stack
    prod_marked: Vec<bool>,                         // production index -> 'is used' flag
}

impl Generator {
    pub fn new() -> Self {
        Self::default()
    }

    fn clear_all(&mut self) {
        self.symbol_min.clear();
        self.prod_min.clear();
        self.best_prod.clear();

        self.clear_axiom();
    }

    fn clear_axiom(&mut self) {
        self.axiom = None;
        self.min_through.clear();
        self.best_parent.clear();

        self.clear_emission();
    }

    fn clear_emission(&mut self) {
        self.which_prod.clear();
        self.on_stack.clear();
        self.prod_marked.clear();
    }

    /// Gathers axiom-independent derivation data.
    ///
    /// Computes shortest derivation paths from productions to
    /// sentences.  For each production stores the computed length.
    /// For each nonterminal stores the ID of its best production
    /// (where 'its' means having that nonterminal on the left).
    pub fn with_grammar(mut self, grammar: &Grammar) -> Self {
        self.clear_all();

        for t in grammar.terminal_ids() {
            self.symbol_min.insert(t, Some(1));
        }

        for nt in grammar.nonterminal_ids() {
            self.symbol_min.insert(nt, None);
            self.best_prod.insert(nt, None);
        }

        self.prod_min.resize(grammar.len(), None);

        loop {
            let mut no_change = true;

            'outer: for (prod_id, prod) in grammar.iter().enumerate() {
                let mut sum = 1;

                for element in prod.rhs() {
                    if let Some(bound) = self.symbol_min[&element] {
                        sum += bound;
                    } else {
                        continue 'outer
                    }
                }

                if self.prod_min[prod_id].map_or(true, |v| sum < v) {
                    self.prod_min[prod_id] = Some(sum);

                    if self.symbol_min[&prod.lhs()].map_or(true, |v| sum < v) {
                        self.symbol_min.insert(prod.lhs(), Some(sum));
                        self.best_prod.insert(prod.lhs(), Some(prod_id));
                        no_change = false;
                    }
                }
            }
            if no_change {
                break
            }
        }
        self
    }

    /// Gathers axiom-specific derivation data.
    ///
    /// Computes shortest derivation paths from `axiom` through all
    /// nonterminals.  For each nonterminal stores the computed length
    /// and the ID of best parent production (where 'parent' means
    /// having that nonterminal on the right).
    pub fn set_axiom(&mut self, grammar: &Grammar, axiom: SymbolID) {
        self.clear_axiom();
        self.axiom = Some(axiom);

        for nt in grammar.nonterminal_ids() {
            self.min_through.insert(nt, None);
            self.best_parent.insert(nt, None);
        }

        self.min_through.insert(axiom, self.symbol_min[&axiom]);

        loop {
            let mut no_change = true;

            for (prod_id, prod) in grammar.iter().enumerate() {
                if let Some(rlen) = self.prod_min[prod_id] {
                    if let Some(dlen) = self.min_through[&prod.lhs()] {
                        if let Some(slen) = self.symbol_min[&prod.lhs()] {
                            let sum = dlen + rlen - slen;

                            for element in prod.rhs_nonterminals() {
                                if self.min_through[element].map_or(true, |v| sum < v) {
                                    self.min_through.insert(*element, Some(sum));
                                    self.best_parent.insert(*element, Some(prod_id));
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
    }

    pub fn emit(&mut self, grammar: &Grammar) /*-> Sentence*/ {
        let mut sentence = Sentence::new();
        let axiom = self.axiom.unwrap();

        self.clear_emission();

        for id in grammar.nonterminal_ids() {
            self.which_prod.insert(id, ProductionUsed::Ready);
            self.on_stack.insert(id, 0);
        }

        self.prod_marked.resize(grammar.len(), false);

        let mut prod_id;

        'outer: loop {
            self.on_stack.insert(axiom, 1);
            let mut nt_id = axiom;

            loop {
                match self.which_prod[&nt_id] {
                    ProductionUsed::Finished => {
                        if nt_id == axiom {
                            break 'outer

                        } else {
                            prod_id = self.use_best_production(nt_id);
                        }
                    }

                    ProductionUsed::ID(id) => {
                        prod_id = id;
                        self.which_prod.insert(nt_id, ProductionUsed::Ready);
                    }

                    _ => {
                        self.choose_productions(grammar);

                        for other_nt_id in grammar.nonterminal_ids() {
                            if other_nt_id == axiom {
                                continue
                            }

                            if let ProductionUsed::ID(_) = self.which_prod[&other_nt_id] {
                                let mut best_prod_id = self.best_parent[&other_nt_id].unwrap();

                                while let Some(prod) = grammar.get(best_prod_id) {
                                    let best_lhs = prod.lhs();

                                    // FIXME why?
                                    if self.on_stack[&best_lhs] == 0 {
                                        break

                                    } else if self.on_stack[&other_nt_id] == 0 {
                                        self.which_prod.insert(best_lhs, ProductionUsed::ID(best_prod_id));
                                        self.prod_marked[best_prod_id] = true;

                                    } else {
                                        self.which_prod.insert(best_lhs, ProductionUsed::Unsure);
                                    }

                                    best_prod_id = self.best_parent[&best_lhs].unwrap();
                                }
                            }
                        }

                        for id in grammar.nonterminal_ids() {
                            if self.which_prod[&id] == ProductionUsed::Ready {
                                self.which_prod.insert(id, ProductionUsed::Finished);
                            }
                        }

                        if nt_id == axiom && self.which_prod[&nt_id] == ProductionUsed::Finished && self.on_stack[&axiom] == 0 {
                            break 'outer

                        } else if let ProductionUsed::ID(id) = self.which_prod[&nt_id] {
                            prod_id = id;
                            self.which_prod.insert(nt_id, ProductionUsed::Ready);

                        } else {
                            prod_id = self.use_best_production(nt_id);
                        }
                    }
                }

                let on_stack = self.on_stack[&nt_id];
                self.on_stack.insert(nt_id, on_stack - 1);

                if let Some(id) = self.update_sentence(grammar, &mut sentence, prod_id) {
                    nt_id = id;
                } else {
                    break
                }
            }
        }
        // sentence
    }

    fn use_best_production(&mut self, nt_id: SymbolID) -> ProductionID {
        self.prod_marked[self.best_prod[&nt_id].unwrap()] = true;

        if self.which_prod[&nt_id] != ProductionUsed::Finished {
            self.which_prod.insert(nt_id, ProductionUsed::Ready);
        }

        self.best_prod[&nt_id].unwrap()
    }

    fn choose_productions(&mut self, grammar: &Grammar) {
        for (prod_id, prod) in grammar.iter().enumerate() {
            if !self.prod_marked[prod_id] {
                match self.which_prod[&prod.lhs()] {
                    ProductionUsed::Ready | ProductionUsed::Unsure => {
                        self.which_prod.insert(prod.lhs(), ProductionUsed::ID(prod_id));
                        self.prod_marked[prod_id] = true;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Returns `SymbolID` of next unresolved nonterminal or `None` if
    /// none remained (end of sentence is reached).
    fn update_sentence(&mut self, grammar: &Grammar, sentence: &mut Sentence, prod_id: ProductionID) -> Option<SymbolID> {
        let prod = grammar.get(prod_id).unwrap();

        for id in prod.rhs() {
            sentence.push(*id);
            if !grammar.is_terminal(*id) {
                let on_stack = self.on_stack[id];
                self.on_stack.insert(*id, on_stack + 1);
            }
        }

        while let Some(id) = sentence.pop() {
            if grammar.is_terminal(id) {
                println!("{}", id);  // FIXME
            } else {
                return Some(id)
            }
        }
        None
    }
}
