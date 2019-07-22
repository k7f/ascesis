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

    pub fn clear(&mut self) {
        self.symbols.clear();
    }

    pub fn push(&mut self, symbol: SymbolID) {
        self.symbols.push(symbol);
    }

    pub fn pop(&mut self) -> Option<SymbolID> {
        self.symbols.pop()
    }

    pub fn as_string(&self, grammar: &Grammar) -> String {
        let mut result = String::new();

        let mut symbols = self.symbols.clone();
        symbols.reverse();

        for id in symbols {
            let symbol = grammar.get_terminal(id).unwrap();

            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(symbol);
        }
        result
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

/// Axiom-independent derivation data.
#[derive(Debug)]
pub struct Generator<'g> {
    grammar:    &'g Grammar,
    symbol_min: HashMap<SymbolID, Option<usize>>, // symbol -> shortest length
    prod_min:   Vec<Option<usize>>,               // production index -> shortest length
    best_prod:  HashMap<SymbolID, Option<usize>>, // nonterminal -> production index
}

impl<'g> Generator<'g> {
    /// Creates a new `Generator` and gathers axiom-independent
    /// derivation data.
    ///
    /// Computes shortest derivation paths from productions to
    /// sentences.  For each production stores the computed length.
    /// For each nonterminal stores the ID of the best production
    /// having that nonterminal on the left.
    pub fn new(grammar: &'g Grammar) -> Self {
        let mut symbol_min = HashMap::new();
        let mut prod_min = Vec::new();
        let mut best_prod = HashMap::new();

        for t in grammar.terminal_ids() {
            symbol_min.insert(t, Some(1));
        }

        for nt in grammar.nonterminal_ids() {
            symbol_min.insert(nt, None);
            best_prod.insert(nt, None);
        }

        prod_min.resize(grammar.len(), None);

        loop {
            let mut no_change = true;

            'outer: for (prod_id, prod) in grammar.iter().enumerate() {
                let mut sum = 1;

                for element in prod.rhs() {
                    if let Some(bound) = symbol_min[&element] {
                        sum += bound;
                    } else {
                        continue 'outer
                    }
                }

                if prod_min[prod_id].map_or(true, |v| sum < v) {
                    prod_min[prod_id] = Some(sum);

                    if symbol_min[&prod.lhs()].map_or(true, |v| sum < v) {
                        symbol_min.insert(prod.lhs(), Some(sum));
                        best_prod.insert(prod.lhs(), Some(prod_id));
                        no_change = false;
                    }
                }
            }
            if no_change {
                break
            }
        }

        for nt in grammar.nonterminal_ids() {
            if best_prod[&nt].is_none() {
                warn!("EMPTY {} (missing base case?)", grammar.get_nonterminal(nt).unwrap());
            }
        }

        Self { grammar, symbol_min, prod_min, best_prod }
    }

    /// Returns a new `RootedGenerator` and gathers axiom-specific
    /// derivation data.
    ///
    /// Computes shortest derivation paths from `axiom` through all
    /// nonterminals.  For each nonterminal stores the computed length
    /// and the ID of the best production having that nonterminal on
    /// the right (best parent production).
    pub fn rooted<S: AsRef<str>>(&self, axiom: S) -> Result<RootedGenerator, String> {
        RootedGenerator::new(self, axiom)
    }
}

/// Axiom-specific derivation data.
#[derive(Debug)]
pub struct RootedGenerator<'b, 'g: 'b> {
    base:        &'b Generator<'g>,
    axiom_id:    SymbolID,
    min_through: HashMap<SymbolID, Option<usize>>, // nonterminal -> shortest length
    best_parent: HashMap<SymbolID, Option<usize>>, // nonterminal -> production index
}

impl<'b, 'g: 'b> RootedGenerator<'b, 'g> {
    fn new<S: AsRef<str>>(base: &'b Generator<'g>, axiom: S) -> Result<Self, String> {
        let axiom = axiom.as_ref();
        let axiom_id = {
            if let Some(id) = base.grammar.id_of_nonterminal(axiom) {
                id
            } else {
                return Err(format!("No such nonterminal: <{}>", axiom))
            }
        };

        let mut min_through = HashMap::new();
        let mut best_parent = HashMap::new();

        for nt in base.grammar.nonterminal_ids() {
            min_through.insert(nt, None);
            best_parent.insert(nt, None);
        }

        min_through.insert(axiom_id, base.symbol_min[&axiom_id]);

        loop {
            let mut no_change = true;

            for (prod_id, prod) in base.grammar.iter().enumerate() {
                if let Some(rlen) = base.prod_min[prod_id] {
                    if let Some(dlen) = min_through[&prod.lhs()] {
                        if let Some(slen) = base.symbol_min[&prod.lhs()] {
                            let sum = dlen + rlen - slen;

                            for element in prod.rhs_nonterminals() {
                                if min_through[element].map_or(true, |v| sum < v) {
                                    min_through.insert(*element, Some(sum));
                                    best_parent.insert(*element, Some(prod_id));
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

        Ok(Self { base, axiom_id, min_through, best_parent })
    }

    pub fn iter<'r>(&'r self) -> Emitter<'r, 'b, 'g> {
        Emitter::new(self)
    }
}

#[derive(Debug)]
pub struct Emitter<'r, 'b: 'r, 'g: 'b> {
    generator:    Option<&'r RootedGenerator<'b, 'g>>,
    which_prod:   HashMap<SymbolID, ProductionUsed>, // nonterminal -> production currently in use
    on_stack:     HashMap<SymbolID, usize>,          // nonterminal -> #occurences on stack
    prod_marked:  Vec<bool>,                         // production index -> 'already used' flag
    in_sentence:  Sentence,
    out_sentence: Sentence,
    num_emitted:  u64,
}

impl<'r, 'b: 'r, 'g: 'b> Emitter<'r, 'b, 'g> {
    fn new(generator: &'r RootedGenerator<'b, 'g>) -> Self {
        let mut which_prod = HashMap::new();
        let mut on_stack = HashMap::new();
        let mut prod_marked = Vec::new();

        for id in generator.base.grammar.nonterminal_ids() {
            which_prod.insert(id, ProductionUsed::Ready);
            on_stack.insert(id, 0);
        }

        prod_marked.resize(generator.base.grammar.len(), false);

        Self {
            generator: Some(generator),
            which_prod,
            on_stack,
            prod_marked,
            in_sentence: Sentence::new(),
            out_sentence: Sentence::new(),
            num_emitted: 0,
        }
    }

    /// For each nonterminal with unassigned production, assign its
    /// first unmarked production.
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
    fn update_sentence(&mut self, grammar: &Grammar, prod_id: ProductionID) -> Option<SymbolID> {
        trace!(
            "PRE input {:?}, production: {}",
            self.in_sentence,
            grammar.get_as_string(prod_id).unwrap()
        );

        let prod = grammar.get(prod_id).unwrap();

        for id in prod.rhs() {
            self.in_sentence.push(*id);
            if grammar.is_nonterminal(*id) {
                let on_stack = self.on_stack[id];
                self.on_stack.insert(*id, on_stack + 1);
            }
        }

        while let Some(id) = self.in_sentence.pop() {
            if grammar.is_terminal(id) {
                self.out_sentence.push(id);
            } else {
                trace!("POST input {:?}", self.in_sentence);
                return Some(id)
            }
        }
        None
    }
}

impl Iterator for Emitter<'_, '_, '_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let generator = self.generator.take().unwrap();
        let grammar = generator.base.grammar;
        let axiom_id = generator.axiom_id;

        self.out_sentence.clear();
        self.on_stack.insert(axiom_id, 1);
        let mut nt_id = axiom_id;
        let mut prod_id;

        loop {
            match self.which_prod[&nt_id] {
                ProductionUsed::Finished => {
                    if nt_id == axiom_id {
                        self.generator = Some(generator);
                        return None
                    } else {
                        prod_id = generator.base.best_prod[&nt_id].unwrap();
                        self.prod_marked[prod_id] = true;
                    }
                }

                ProductionUsed::ID(id) => {
                    prod_id = id;
                    self.which_prod.insert(nt_id, ProductionUsed::Ready);
                }

                ProductionUsed::Ready | ProductionUsed::Unsure => {
                    self.choose_productions(grammar);

                    // For any non-axiom NT in use and not yet on
                    // stack, use all its best ancestors up until
                    // there is one already in use.  For any non-axiom
                    // NT in use and on stack, mark all its best
                    // ancestors as unsure up until there is one
                    // already in use.
                    for child_nt_id in grammar.nonterminal_ids() {
                        if child_nt_id == axiom_id {
                            continue
                        }

                        if let ProductionUsed::ID(_) = self.which_prod[&child_nt_id] {
                            let mut parent_nt_id = child_nt_id;

                            while let Some(best_prod_id) = generator.best_parent[&parent_nt_id] {
                                parent_nt_id = grammar.get(best_prod_id).unwrap().lhs();

                                if let ProductionUsed::ID(_) = self.which_prod[&parent_nt_id] {
                                    break
                                } else if self.on_stack[&child_nt_id] == 0 {
                                    self.which_prod
                                        .insert(parent_nt_id, ProductionUsed::ID(best_prod_id));
                                    self.prod_marked[best_prod_id] = true;
                                } else {
                                    self.which_prod.insert(parent_nt_id, ProductionUsed::Unsure);
                                }
                            }
                        }
                    }

                    for id in grammar.nonterminal_ids() {
                        if self.which_prod[&id] == ProductionUsed::Ready {
                            self.which_prod.insert(id, ProductionUsed::Finished);
                        }
                    }

                    if nt_id == axiom_id
                        && self.which_prod[&nt_id] == ProductionUsed::Finished
                        && self.on_stack[&axiom_id] == 0
                    {
                        self.generator = Some(generator);
                        return None
                    } else if let ProductionUsed::ID(id) = self.which_prod[&nt_id] {
                        prod_id = id;
                        self.which_prod.insert(nt_id, ProductionUsed::Ready);
                    } else {
                        prod_id = generator.base.best_prod[&nt_id].unwrap();
                        self.prod_marked[prod_id] = true;

                        if self.which_prod[&nt_id] != ProductionUsed::Finished {
                            self.which_prod.insert(nt_id, ProductionUsed::Ready);
                        }
                    }
                }
            }

            let on_stack = self.on_stack[&nt_id];
            self.on_stack.insert(nt_id, on_stack - 1);

            if let Some(id) = self.update_sentence(grammar, prod_id) {
                nt_id = id;
            } else {
                break
            }
        }

        self.num_emitted += 1;
        let result = self.out_sentence.as_string(grammar);
        debug!("{}. {}", self.num_emitted, result);

        self.generator = Some(generator);
        Some(result)
    }
}
