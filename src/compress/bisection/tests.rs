
use super::*;
use crate::compress::test;

#[test]
fn grammar_correct() {
    test::common::grammar_correct(bisection);
}

#[test]
fn grammar_correct_large_input() {
    test::common::grammar_correct_large_input(bisection);
}

#[test]
fn input_len_0() {
    test::common::input_len_0(bisection);
}

#[test]
fn input_len_1() {
    test::common::input_len_1(bisection);
}

#[test]
fn nonterminals_in_bounds() {
    test::common::nonterminals_in_bounds(bisection);
}

#[test]
fn produce_one_rule(){
    let input = b"ab";
    let (grammar, start_rule) = bisection(input);
    assert_eq!(grammar.len(), 1);
    let l = grammar[0].expansion[0];
    let r = grammar[0].expansion[1];        
    assert_eq!(l as u8, b'a');
    assert_eq!(r as u8, b'b');

    assert_eq!(start_rule.len(), 1);
    assert_eq!(start_rule[0], 256 as GSize);
}

#[test]
fn reuse_existing_rules(){
    let input = b"abcdabcd";
    
    let (grammar, start_rule) = bisection(input);
    assert_eq!(grammar.len(), 4);
    assert_eq!(start_rule.len(), 1);
    
    let start_symbol = start_rule[0];
    let fourth_rule_1 = grammar[3].expansion[0];
    let fourth_rule_2 = grammar[3].expansion[1];
    assert_eq!(fourth_rule_1, 258 as GSize);
    assert_eq!(fourth_rule_2, 258 as GSize);

    
    assert_eq!(start_symbol, 259 as GSize);
}

#[test]
fn dont_reuse_existing_rules(){
    let input = b"abcdabce";
    
    let (grammar, start_rule) = bisection(input);
    assert!(start_rule[0] > 256);

    let id: usize = (start_rule[0] - 256) as usize;
    let rule_part_1 = grammar[id].expansion[0];
    let rule_part_2 = grammar[id].expansion[1];
    assert_ne!(rule_part_1, rule_part_2);
}


#[test]
fn test_deterministic() {
    let input = b"abcdabcd";
    let (grammar_1, start_rule_1) = bisection(input);
    let (grammar_2, start_rule_2) = bisection(input);
    assert_eq!(start_rule_1, start_rule_2);
    assert_eq!(grammar_1, grammar_2);
}
