use crate::utils::{GSize, Rule};
use std::collections::HashMap;

//Sym = GSize
type RuleId = usize;
type NodeId = usize;
type Digram = (GSize, GSize);

const GS: GSize = GSize::MAX;

fn is_terminal(s: GSize) -> bool { s < 256}

fn is_nonterminal(s: GSize) -> bool {s >= 256}

fn rule_to_nt(rule: RuleId) -> GSize{
    return 256 + (rule as GSize - 1)
}

fn nt_to_rule(sym: GSize) -> RuleId{
    return (sym - 256 + 1) as RuleId
}

#[derive(Debug)]
pub struct Node {
    pub sym: GSize,
    pub prev: NodeId,
    pub next: NodeId,
    pub rule: Option<RuleId>,
    pub occ_prev: NodeId,
    pub occ_next: NodeId,
    pub deleted: bool,
    pub queued: bool
}

#[derive(Debug)]
pub struct SRule {
    pub guard: NodeId,
    pub occurences: usize,
    pub first_occ: Option<NodeId>,
    pub queued: bool
}

#[derive(Debug)]
pub struct Grammar {
    pub nodes: Vec<Node>,
    pub rules: Vec<SRule>
}

impl Grammar {
    pub fn new() -> Self {
        let mut g = Self { nodes: Vec::new(), rules: Vec::new() };
        g.new_rule();
        return g
    }

    fn alloc_node(&mut self, sym: GSize) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node { sym, prev: id, next: id, rule: None, occ_prev: id, occ_next: id, deleted: false, queued: false});
        return id
    }

    fn new_rule(&mut self) -> RuleId {
        let guard = self.alloc_node(GS);
        self.nodes[guard].prev = guard;
        self.nodes[guard].next = guard;
        
        let rule_id = self.rules.len();
        self.rules.push(SRule { guard, occurences: 0, first_occ: None, queued: false });

        self.nodes[guard].rule = Some(rule_id);
        
        return rule_id;
    }

    pub fn insert_before(&mut self, next_node: NodeId, sym: GSize) -> NodeId {
        let new_id = self.alloc_node(sym);
        let prev = self.nodes[next_node].prev;

        self.nodes[new_id].prev = prev;
        self.nodes[new_id].next = next_node;
        self.nodes[next_node].prev = new_id;
        self.nodes[prev].next = new_id;

        return new_id
    }

    pub fn insert_back(&mut self, rule: RuleId, sym: GSize) -> NodeId{
        let guard = self.rules[rule].guard;
        let n_id = self.insert_before(guard, sym);
        return n_id
    }

    pub fn remove(&mut self, n_id: NodeId) {
        self.nodes[n_id].deleted = true;
        self.nodes[n_id].prev = n_id;
        self.nodes[n_id].next = n_id;
    }

    pub fn occ_insert(&mut self, rule_id: RuleId, node: NodeId) {

        self.nodes[node].occ_prev = node;
        self.nodes[node].occ_next = node;

        match self.rules[rule_id].first_occ {
            None => {
                self.rules[rule_id].first_occ = Some(node);
            }
            Some(head) => {
                let tail = self.nodes[head].occ_prev;

                self.nodes[node].occ_next = head;
                self.nodes[node].occ_prev = tail;
                self.nodes[tail].occ_next = node;
                self.nodes[head].occ_prev = node;
            }
        }

        self.rules[rule_id].occurences += 1;
    }

    pub fn occ_remove(&mut self, rule_id: RuleId, node: NodeId) {

        let prev = self.nodes[node].occ_prev;
        let next = self.nodes[node].occ_next;

        if prev == node && next == node{
            self.rules[rule_id].first_occ = None;
        } else{
            self.nodes[prev].occ_next = next;
            self.nodes[next].occ_prev = prev;

            if self.rules[rule_id].first_occ == Some(node){
                self.rules[rule_id].first_occ = Some(next);
            }
        }

        self.nodes[node].occ_next = node;
        self.nodes[node].occ_prev = node;

        self.rules[rule_id].occurences -= 1;
    }

    pub fn get_only_occurence(&self, rule: RuleId)-> NodeId{
        return self.rules[rule].first_occ.unwrap();
    }

    pub fn replace_digram(&mut self, digram_pos: NodeId, next_symbol: GSize) -> NodeId {
        let digram_right_pos = self.nodes[digram_pos].next;

        let prev = self.nodes[digram_pos].prev;
        let next = self.nodes[digram_right_pos].next;

        let nonterminal_node = self.alloc_node(next_symbol);

        self.nodes[nonterminal_node].prev = prev;
        self.nodes[nonterminal_node].next = next;
        self.nodes[prev].next = nonterminal_node;
        self.nodes[next].prev = nonterminal_node;

        //Old nodes detached
        self.remove(digram_pos);
        self.remove(digram_right_pos);

        return nonterminal_node
    }

    pub fn is_guard(&self, n_id:NodeId) -> bool{
        return self.nodes[n_id].sym == GSize::MAX
    }

    fn get_digram(&self, position: NodeId) -> Option<(GSize, GSize)> {
        let digram_right = self.nodes[position].next;
        if self.is_guard(position) || self.is_guard(digram_right){
            return None;
        }else {
            return Some((self.nodes[position].sym, self.nodes[digram_right].sym));
        }
    }

    fn rule_alive(&self, rule: RuleId) -> bool{
        let guard = self.rules[rule].guard;
        return self.nodes[guard].next != guard;
    }

}

fn push_queue_node(grammar: &mut Grammar, nodes_to_check: &mut Vec<NodeId>, node: NodeId) {
    if grammar.is_guard(node){return;} 
    if grammar.nodes[node].queued{return;}
    if grammar.nodes[node].deleted{return;}
    
    grammar.nodes[node].queued = true;
    nodes_to_check.push(node);
}

fn push_queue_rule(grammar: &mut Grammar, rules_to_check: &mut Vec<RuleId>, rule_id: RuleId){
    if rule_id == 0 {return;}
    if grammar.rules[rule_id].occurences != 1{return;}
    if grammar.rules[rule_id].queued {return;}
    grammar.rules[rule_id].queued = true;
    rules_to_check.push(rule_id);
}
    
fn remove_rule(grammar: &mut Grammar, 
    index: &mut HashMap<Digram, NodeId>, 
    rule_id: RuleId, 
    nodes_to_check: &mut Vec<NodeId>){
    let remaining_node = grammar.get_only_occurence(rule_id);

    let prev = grammar.nodes[remaining_node].prev;
    let next = grammar.nodes[remaining_node].next;

    let rule_guard = grammar.rules[rule_id].guard;
    let first = grammar.nodes[rule_guard].next;
    let last = grammar.nodes[rule_guard].prev;

    remove_from_index(index, grammar, prev);
    remove_from_index(index, grammar, remaining_node);

    grammar.occ_remove(rule_id, remaining_node);

    grammar.nodes[prev].next = first;
    grammar.nodes[first].prev = prev;

    grammar.nodes[last].next = next;
    grammar.nodes[next].prev = last;

    grammar.remove(remaining_node);
    grammar.nodes[rule_guard].next = rule_guard;
    grammar.nodes[rule_guard].prev = rule_guard;
    push_queue_node(grammar, nodes_to_check, prev);
    push_queue_node(grammar, nodes_to_check, last);
        
}

fn remove_from_index(index: &mut HashMap<(GSize, GSize), NodeId>, grammar: &Grammar, position: NodeId){
    if let Some(d) = grammar.get_digram(position){
        if index.get(&d) == Some(&position) {
            index.remove(&d);
        }
    }
}

fn add_to_index(index: &mut HashMap<(GSize, GSize), NodeId>, grammar: &Grammar, position: NodeId) -> Option<NodeId>{
    let d = grammar.get_digram(position)?;
    if let Some(&other) = index.get(&d) {
        return Some(other);
    }else {
        index.insert(d, position);
        return None;
    }
}

fn substitute_digram(grammar: &mut Grammar,
    index: &mut HashMap<(GSize, GSize), NodeId>,
    digram_first_part: NodeId,
    nt_symbol: GSize,
    nodes_to_check: &mut Vec<NodeId>,
    rules_to_check: &mut Vec<RuleId>
    ){
    let left_node = grammar.nodes[digram_first_part].prev;
    let digram_right_part = grammar.nodes[digram_first_part].next;

    let first_sym = grammar.nodes[digram_first_part].sym;
    let second_sym = grammar.nodes[digram_right_part].sym;

    if is_nonterminal(first_sym){
        let rule_id = nt_to_rule(first_sym);
        grammar.occ_remove(rule_id, digram_first_part);
        push_queue_rule(grammar, rules_to_check, rule_id);
    }

    if is_nonterminal(second_sym){
        let rule_id = nt_to_rule(second_sym);
        grammar.occ_remove(rule_id, digram_right_part);
        push_queue_rule(grammar, rules_to_check, rule_id);
    }

    remove_from_index(index, grammar, left_node);
    remove_from_index(index, grammar, digram_first_part);
    remove_from_index(index, grammar, digram_right_part);

    let nt_node = grammar.replace_digram(digram_first_part, nt_symbol);

    let used_rule = nt_to_rule(nt_symbol);
    grammar.occ_insert(used_rule, nt_node);


    add_to_index(index, grammar, left_node);
    add_to_index(index, grammar, nt_node);
    push_queue_node(grammar, nodes_to_check, left_node);
    push_queue_node(grammar, nodes_to_check, nt_node);
}

fn insert_rule (
    grammar: &mut Grammar, 
    index: &mut HashMap<(GSize, GSize), NodeId>,
    digram: (GSize, GSize),
    rule_id: RuleId){

    let rhs_1 = grammar.alloc_node(digram.0);
    let rhs_2 = grammar.alloc_node(digram.1);
    let guard = grammar.rules[rule_id].guard;

    if is_nonterminal(digram.0) {
        let rule_id = nt_to_rule(digram.0);
        grammar.occ_insert(rule_id, rhs_1);
    
    }

    if is_nonterminal(digram.1) {
        let rule_id = nt_to_rule(digram.1);
        grammar.occ_insert(rule_id, rhs_2);

    }
    
    grammar.nodes[guard].prev = rhs_2;
    grammar.nodes[guard].next = rhs_1;
    grammar.nodes[rhs_1].prev = guard;
    grammar.nodes[rhs_1].next = rhs_2;
    grammar.nodes[rhs_2].prev = rhs_1;
    grammar.nodes[rhs_2].next = guard;
    add_to_index(index,grammar, rhs_1);
}

fn enforce_digram_uniqueness(
    grammar: &mut Grammar, 
    index: &mut HashMap<(GSize, GSize), NodeId>, 
    pos: NodeId,
    nodes_to_check: &mut Vec<NodeId>,
    rules_to_check: &mut Vec<RuleId>
) {

    if grammar.is_guard(pos) {
        return;
    }
    let Some(digram) = grammar.get_digram(pos) else {
        return;
    };
    let Some(&existing) = index.get(&digram) else {
        add_to_index(index, grammar, pos);
        return;
    };
    if grammar.nodes[pos].deleted {
        return;
    }

    if existing == pos { return; }
    if grammar.nodes[existing].next == pos || grammar.nodes[pos].next == existing {
        return;
    }

    let g = grammar.nodes[existing].prev;

    let second_symbol = grammar.nodes[existing].next;
    let is_len_2 = grammar.nodes[second_symbol].next == g;
    
    if grammar.is_guard(g) && is_len_2{
        let rule_id= grammar.nodes[g].rule.unwrap();        

        let nt_symbol = rule_to_nt(rule_id);
        substitute_digram(grammar, index, pos, nt_symbol, nodes_to_check, rules_to_check);

        return;
    }
    let rule_id = grammar.new_rule();
    let nt = rule_to_nt(rule_id);

    substitute_digram(grammar, index, existing, nt, nodes_to_check, rules_to_check);
    substitute_digram(grammar, index, pos, nt, nodes_to_check, rules_to_check);
    insert_rule(grammar, index, digram, rule_id);

}

fn enforced_rule_utility(
    grammar: &mut Grammar, 
    index: &mut HashMap<(GSize, GSize), NodeId>, 
    rules_to_check: &mut Vec<RuleId>, 
    nodes_to_check: &mut Vec<NodeId>){
    while let Some(rule) = rules_to_check.pop() {
        grammar.rules[rule].queued = false;
        if rule != 0 && grammar.rules[rule].occurences == 1 {
            remove_rule(grammar, index, rule, nodes_to_check);
        }
    }
}

fn collect_right_side(grammar: &Grammar, rid: RuleId) -> Vec<GSize> {
    let guard = grammar.rules[rid].guard;
    let mut current = grammar.nodes[guard].next;
    let mut right = Vec::new();

    while current != guard{
        if !grammar.nodes[current].deleted{
            right.push(grammar.nodes[current].sym);
        }
        current = grammar.nodes[current].next;
    }
    return right;
}

fn remap_rules(symbol: GSize, old_to_new: &[Option<RuleId>]) -> GSize{
    if is_terminal(symbol){
        return symbol;
    }
    let old_rid = nt_to_rule(symbol);
    let new_rid = old_to_new[old_rid].unwrap();
    return rule_to_nt(new_rid);
}

fn export_grammar(grammar: &Grammar) -> (Vec<Rule>, Vec<GSize>) {
    let old_rules = collect_topological_rules(grammar);

    let mut old_to_new = vec![None; grammar.rules.len()];
    let mut rhs_map = vec![Vec::new(); grammar.rules.len()];

    let mut next_id: RuleId = 1;

    for &old in &old_rules {
        let rhs = collect_right_side(grammar, old);

        if rhs.len() < 2 {
            panic!("Rule with length 0 or 1, this should never happen");
        }

        let helper_count = rhs.len().saturating_sub(2);
        rhs_map[old] = rhs;

        next_id += helper_count;
        old_to_new[old] = Some(next_id);
        next_id += 1;
    }

    let mut binary_grammar = vec![Rule { expansion: [0, 0] }; next_id - 1];

    let mut start_rule = Vec::new();
    let guard = grammar.rules[0].guard;
    let mut node = grammar.nodes[guard].next;

    while node != guard {
        if !grammar.nodes[node].deleted {
            start_rule.push(remap_rules(grammar.nodes[node].sym, &old_to_new));
        }
        node = grammar.nodes[node].next;
    }

    for &old in &old_rules {
        let new_id = old_to_new[old].unwrap();
        let mut rhs: Vec<GSize> = Vec::with_capacity(rhs_map[old].len());

        for &sym in &rhs_map[old] {
            let mapped = remap_rules(sym, &old_to_new);
            rhs.push(mapped);
        }

        if rhs.len() == 2{
            binary_grammar[new_id - 1] = Rule {
                expansion: [rhs[0], rhs[1]],
            };
        }else {
            let n = rhs.len();
            let helper_count = n - 2;
            let first_helper = new_id - helper_count;

            binary_grammar[first_helper - 1] = Rule {
                expansion: [rhs[n - 2], rhs[n - 1]],
            };

            for t in 1..helper_count {
                let rid = first_helper + t;

                binary_grammar[rid - 1] = Rule {
                    expansion: [rhs[n - 2 - t], rule_to_nt(rid - 1)],
                };
            }

            binary_grammar[new_id - 1] = Rule {
                expansion: [rhs[0], rule_to_nt(new_id - 1)],
            };
        }
    }

    (binary_grammar, start_rule)
}

fn dfs(
    grammar: &Grammar,
    rid: RuleId,
    seen: &mut [u8],
    order: &mut Vec<RuleId>,
    ) {
    if !grammar.rule_alive(rid) {
        return;
    }
    if seen[rid] == 2{
        return
    }
    seen[rid] = 1;
    for sym in collect_right_side(grammar, rid) {
        if is_nonterminal(sym) {
            dfs(grammar, nt_to_rule(sym), seen, order);
        }
    }

    seen[rid] = 2;
    order.push(rid);
}

fn collect_topological_rules(grammar: &Grammar) -> Vec<RuleId> {
    let mut seen = vec![0u8; grammar.rules.len()];
    let mut order = Vec::new();
    let guard = grammar.rules[0].guard;
    let mut node = grammar.nodes[guard].next;

    while node != guard {
        if !grammar.nodes[node].deleted {
            let sym = grammar.nodes[node].sym;

            if is_nonterminal(sym) {
                dfs(grammar, nt_to_rule(sym), &mut seen, &mut order);
            }
        }

        node = grammar.nodes[node].next;
    }
    order
}


pub fn sequitur_internal(file_content: &[u8]) -> Grammar {
    let mut grammar = Grammar::new();
    let mut index: HashMap<Digram, NodeId> =HashMap::new();
    let s: RuleId = 0;
    let mut nodes_to_check: Vec<NodeId> = Vec::new();
    let mut rules_to_check: Vec<RuleId> = Vec::new();

    for &sym in file_content{
        let new_node = grammar.insert_back(s, sym as GSize);
        let digram_left_part = grammar.nodes[new_node].prev;

        push_queue_node(&mut grammar, &mut nodes_to_check, digram_left_part);

        while let Some(node) = nodes_to_check.pop() {
            grammar.nodes[node].queued = false;
            if grammar.nodes[node].deleted && !grammar.is_guard(node) {
                continue;
            }
            enforce_digram_uniqueness(&mut grammar, &mut index, node, &mut nodes_to_check, &mut rules_to_check);

            enforced_rule_utility(&mut grammar, &mut index, &mut rules_to_check, &mut nodes_to_check);
        }
    }
    return grammar;
}

pub fn sequitur(file_content: &[u8]) -> (Vec<Rule>, Vec<GSize>) {    
    let grammar = sequitur_internal(file_content);
    return export_grammar(&grammar);
}

#[cfg(test)]
mod tests;
