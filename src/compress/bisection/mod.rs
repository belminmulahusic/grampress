use std::{collections::HashMap, vec};
use crate::utils::{Rule, GSize};

const HASH_BASE: u64 = 85263221;

fn subhash(prefix: &[u64], power: &[u64], l: usize, r: usize) -> u64 {
    return prefix[r].wrapping_sub(prefix[l].wrapping_mul(power[r - l]));
}

pub fn bisection(input: &[u8]) -> (Vec<Rule>, Vec<GSize>) {

    let len = input.len();
    let mut memory: HashMap<(u32, u64), (usize, usize, GSize)> = HashMap::new();
    let mut grammar: Vec<Rule> = Vec::new();

    let mut power = vec![0u64; len + 1];
    let mut prefix = vec![0u64; len + 1];
    let mut start_rule: Vec<GSize> = Vec::new();

    if len == 0{
        return (grammar, start_rule);
    }


    power[0] = 1;
    for i in 0..len{
        power[i + 1] = power[i].wrapping_mul(HASH_BASE);
        prefix[i + 1] = prefix[i].wrapping_mul(HASH_BASE).wrapping_add(input[i] as u64 + 1);
    }
    start_rule.push(build_grammar(input, &prefix, &power, 0, len, &mut memory, &mut grammar));
    return (grammar, start_rule);
}

fn build_grammar(
    input: &[u8], 
    prefix: &[u64], 
    power: &[u64], 
    l: usize, 
    r: usize,
    memory: &mut HashMap<(u32, u64), (usize, usize, GSize)>,
    grammar: &mut Vec<Rule>) -> GSize{
        let len: usize = r - l;
        if len == 1{
            return input[l] as GSize;
        }

        let key = (len as u32, subhash(prefix, power, l, r));
        if let Some(&(l2, r2, sym)) = memory.get(&key) {
            if &input[l..r] == &input[l2..r2] {
                return sym;
            }
        }
        let m = l + (len.next_power_of_two()>>1);
        let left = build_grammar(input, prefix, power, l, m, memory, grammar);
        let right = build_grammar(input, prefix, power, m, r, memory, grammar);

        let symbol = (grammar.len() + 256) as GSize;
        grammar.push(Rule { expansion: [left, right] });

        memory.insert(key, (l, r, symbol));
        return symbol;

    }
#[cfg(test)]
mod tests;
