use crate::utils::{GSize, Rule};
use indexmap::IndexSet;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone, Copy)]
struct SymbolRecord {
    value: Option<GSize>,
    left: Option<usize>,
    right: Option<usize>,
}

fn initialize_symbol_record(input: &[u8]) -> Vec<SymbolRecord> {
    let mut symbols: Vec<SymbolRecord> = Vec::with_capacity(input.len());

    for (i, &sym) in input.iter().enumerate() {
        symbols.push(SymbolRecord {
            value: Some(sym as GSize),
            left: if i > 0 { Some(i - 1) } else { None },
            right: if i < input.len() - 1 {
                Some(i + 1)
            } else {
                None
            },
        });
    }
    return symbols;
}

fn collect_pairs(symbols: &Vec<SymbolRecord>) -> HashMap<(GSize, GSize), HashSet<usize>> {
    let mut active_pairs: HashMap<(GSize, GSize), HashSet<usize>> = HashMap::new();

    for i in 0..symbols.len() - 1 {
        if let (Some(left), Some(right)) = (symbols[i].value, symbols[i + 1].value) {
            let pair = (left, right);
            active_pairs.entry(pair).or_default().insert(i);
        }
    }
    return active_pairs;
}

fn initialize_prioritiy_queue(
    active_pairs: &HashMap<(GSize, GSize), HashSet<usize>>,
    sqrt_n: usize,
) -> (Vec<IndexSet<(GSize, GSize)>>, usize) {
    let mut priority_queue: Vec<IndexSet<(GSize, GSize)>> = vec![IndexSet::new(); sqrt_n];
    let mut count = 0;
    for (pair, positions) in active_pairs {
        if positions.len() >= sqrt_n {
            priority_queue[sqrt_n - 1].insert(*pair);
        } else {
            priority_queue[positions.len() - 1].insert(*pair);
        }
        if count < positions.len() {
            count = positions.len();
        }
    }
    return (priority_queue, count);
}

fn get_most_frequent_pair(
    priority_queue: &mut Vec<IndexSet<(GSize, GSize)>>,
    max_pair_count: &mut usize,
    sqrt_n: usize,
    active_pairs: &HashMap<(GSize, GSize), HashSet<usize>>
) -> Option<(GSize, GSize)> {
    while *max_pair_count > 0 && priority_queue[*max_pair_count - 1].is_empty() {
        *max_pair_count -= 1;
    }
    if *max_pair_count < 2 {
        return None;
    }

    let bucket_id = *max_pair_count - 1;

    if bucket_id == sqrt_n - 1{
        let mut best_pair: Option<(GSize, GSize)> = None;
        let mut best_count = 0;

        for &pair in priority_queue[bucket_id].iter(){
            if let Some(positions) = active_pairs.get(&pair){
                let count = positions.len();
                if count > best_count{
                    best_count = count;
                    best_pair = Some(pair);
                }
            }
        }

        if let Some(pair) = best_pair {
            priority_queue[bucket_id].swap_remove(&pair);
            return Some(pair);
        }
    }

    let value = match priority_queue[bucket_id].iter().next().copied() {
        Some(v) => v,
        None => return None,
    };
    priority_queue[*max_pair_count - 1].swap_remove(&value);

    return Some(value);
}

fn get_correct_bucket(sqrt_n: usize, count: usize) -> usize {
    if count < sqrt_n {
        return count - 1;
    }
    return sqrt_n - 1;
}

fn decrement_pair(
    pair: (GSize, GSize),
    pos: usize,
    active_pairs: &mut HashMap<(GSize, GSize), HashSet<usize>>,
    priority_queue: &mut Vec<IndexSet<(GSize, GSize)>>,
    sqrt_n: usize,
) {
    if let Some(positions) = active_pairs.get_mut(&pair) {
        let old_count = positions.len();
        positions.remove(&pos);
        let new_count = positions.len();

        if old_count > 1 {
            let old_bucket = get_correct_bucket(sqrt_n, old_count);
            priority_queue[old_bucket].swap_remove(&pair);
            let new_bucket = get_correct_bucket(sqrt_n, new_count);
            priority_queue[new_bucket].insert(pair);
        }
        if new_count == 0 {
            let old_bucket = get_correct_bucket(sqrt_n, old_count);
            priority_queue[old_bucket].swap_remove(&pair);
            active_pairs.remove(&pair);
        }
    }
}

fn increment_pair(
    pair: (GSize, GSize),
    pos: usize,
    active_pairs: &mut HashMap<(GSize, GSize), HashSet<usize>>,
    priority_queue: &mut Vec<IndexSet<(GSize, GSize)>>,
    sqrt_n: usize,
) {
    let positions = active_pairs.entry(pair).or_default();
    let old_count = positions.len();
    positions.insert(pos);
    let new_count = positions.len();

    if old_count > 1 {
        let old_bucket = get_correct_bucket(sqrt_n, old_count);
        priority_queue[old_bucket].swap_remove(&pair);
    }

    if new_count > 1 {
        let new_bucket = get_correct_bucket(sqrt_n, new_count);
        priority_queue[new_bucket].insert(pair);
    }
}


fn replace_pair(
    symbols: &mut Vec<SymbolRecord>,
    active_pairs: &mut HashMap<(GSize, GSize), HashSet<usize>>,
    priority_queue: &mut Vec<IndexSet<(GSize, GSize)>>,
    pair: (GSize, GSize),
    sqrt_n: usize,
    next_symbol: GSize,
) {
    let pair_positions: Vec<usize> = active_pairs
        .remove(&pair)
        .unwrap()
        .iter()
        .copied()
        .collect();
    for pos in pair_positions {
        if symbols[pos].value == None {
            continue;
        }
        let right_pair_part = symbols[pos].right.unwrap();
        let pair_to_be_replaced = (
            symbols[pos].value.unwrap(),
            symbols[right_pair_part].value.unwrap(),
        );
        if pair_to_be_replaced != pair {
            continue;
        }

        if symbols[pos].left != None {
            let left_neighbor_pos = symbols[pos].left.unwrap();
            let left_neighbor = symbols[left_neighbor_pos]; //Abcd 
            let old_left_pair = (left_neighbor.value.unwrap(), pair.0); //ABcd
            decrement_pair(
                old_left_pair,
                left_neighbor_pos,
                active_pairs,
                priority_queue,
                sqrt_n,
            );

            let new_left_pair = (left_neighbor.value.unwrap(), next_symbol);
            increment_pair(
                new_left_pair,
                left_neighbor_pos,
                active_pairs,
                priority_queue,
                sqrt_n,
            );
        }

        if symbols[right_pair_part].right != None {
            let right_neighbor_pos = symbols[right_pair_part].right.unwrap();
            let right_neighbor = symbols[right_neighbor_pos].value.unwrap();
            let old_right_pair = (symbols[right_pair_part].value.unwrap(), right_neighbor); //abCD
            decrement_pair(
                old_right_pair,
                right_pair_part,
                active_pairs,
                priority_queue,
                sqrt_n,
            );

            let new_right_pair = (next_symbol, right_neighbor);
            increment_pair(new_right_pair, pos, active_pairs, priority_queue, sqrt_n);
        }
        symbols[pos].value = Some(next_symbol);
        symbols[right_pair_part].value = None;
        symbols[pos].right = symbols[right_pair_part].right;

        if let Some(next_next_pos) = symbols[right_pair_part].right {
            symbols[next_next_pos].left = Some(pos);
        }
    }
}

fn retrieve_start_rule(symbols: &Vec<SymbolRecord>) -> Vec<GSize> {
    let mut start_rule: Vec<GSize> = Vec::new();
    for s in symbols {
        if s.value != None {
            start_rule.push(s.value.unwrap());
        }
    }
    return start_rule;
}

pub fn repair(file_content: &[u8]) -> (Vec<Rule>, Vec<GSize>) {
    let n = file_content.len();
    if n == 0{
        return (Vec::new(), Vec::new());
    }
    let mut grammar: Vec<Rule> = Vec::new();
    let mut symbols = initialize_symbol_record(file_content);
    let mut active_pairs = collect_pairs(&symbols);

    let sqrt_n = (n as f64).sqrt().ceil() as usize;
    let (mut priority_queue, mut max_pair_count) =
        initialize_prioritiy_queue(&active_pairs, sqrt_n);

    let mut next_symbol = 256;
    loop {
        max_pair_count = max_pair_count.min(sqrt_n);
        let pair = get_most_frequent_pair(&mut priority_queue, 
            &mut max_pair_count, 
            sqrt_n, 
            &active_pairs);        
        match pair {
            Some(p) => {
                replace_pair(
                    &mut symbols,
                    &mut active_pairs,
                    &mut priority_queue,
                    p,
                    sqrt_n,
                    next_symbol,
                );
                grammar.push(Rule {
                    expansion: [p.0, p.1],
                });
                next_symbol += 1;
            }
            None => {
                break;
            }
        }
    }
    let start_rule = retrieve_start_rule(&symbols);
    (grammar, start_rule)
}

#[cfg(test)]
mod tests;
