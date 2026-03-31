   use super::*;
    use crate::compress::test;

    #[test]
    fn grammar_correct() {
        test::common::grammar_correct(sequitur);
    }

    #[test]
    fn grammar_correct_large_input() {
        test::common::grammar_correct_large_input(sequitur);
    }

    #[test]
    fn input_len_0() {
        test::common::input_len_0(sequitur);
    }

    #[test]
    fn input_len_1() {
        test::common::input_len_1(sequitur);
    }

    #[test]
    fn nonterminals_in_bounds() {
        test::common::nonterminals_in_bounds(sequitur);
    }

    #[test]
    fn start_rule_alive_nonempty_input() {
        let input = b"abcdabcfabababaabcde";
        let grammar = sequitur_internal(input);

        assert!(
            grammar.rule_alive(0));
    }

    #[test]
    fn rule_alive_has_nonempty_rhs() {
        let input = b"abcdabcfabababaabcde";
        let grammar = sequitur_internal(input);

        for id in 0..grammar.rules.len() {
            if grammar.rule_alive(id) {
                let guard = grammar.rules[id].guard;
                let first = grammar.nodes[guard].next;
                assert_ne!(first, guard);
            }
        }
    }

    #[test]
    fn dead_rule_has_empty_rhs() {
        let input = b"abcdabcfabababaabcde";
        let grammar = sequitur_internal(input);

        for id in 0..grammar.rules.len() {
            if !grammar.rule_alive(id) {
                let guard = grammar.rules[id].guard;
                let first = grammar.nodes[guard].next;
                assert_eq!(first, guard);
            }
        }
    }

   #[test]
    fn dead_rule_are_unused() {
        let input = b"abcdabcfabababaabcde";
        let grammar = sequitur_internal(input);

        for id in 1..grammar.rules.len() {
            if !grammar.rule_alive(id) {
                let occurences = grammar.rules[id].occurences;
                assert_eq!(occurences, 0);
            }
        }
    }

    #[test]
    fn alive_rules_are_used(){
        let input = b"abcdabcfabababaabcde";
        let grammar = sequitur_internal(input);
        let mut used_rules: Vec<RuleId> = Vec::new();

        let start_guard = grammar.rules[0].guard;
        let mut next = grammar.nodes[start_guard].next;

        while next != start_guard{
            let sym = grammar.nodes[next].sym;
            if is_nonterminal(sym){
                used_rules.push(nt_to_rule(sym));
            }
            next = grammar.nodes[next].next;
        }

        for id in 1..grammar.rules.len(){
            let guard = grammar.rules[id].guard;
            let mut current = grammar.nodes[guard].next;
            while current != guard{
                let sym = grammar.nodes[current].sym;
                if is_nonterminal(sym){
                    used_rules.push(nt_to_rule(sym));
                }
                current = grammar.nodes[current].next;
            }
        }

        for id in 1..grammar.rules.len(){
            if grammar.rule_alive(id){
                assert!(used_rules.contains(&id));
            }
        }
    }

    #[test]
    fn digram_uniqueness() {
        let input = b"abcdabcfabababaabcde";
        let grammar = sequitur_internal(input);

        let mut digrams: HashMap<(GSize, GSize), (usize, usize)> = HashMap::new();

        for id in 0..grammar.rules.len() {
            let guard = grammar.rules[id].guard;
            let mut pos = grammar.nodes[guard].next;

            while pos != guard {
                if grammar.nodes[pos].deleted {
                    pos = grammar.nodes[pos].next;
                    continue;
                }

                let next = grammar.nodes[pos].next;
                if next == guard  {
                    pos = grammar.nodes[pos].next;
                    continue;
                }

                let digram = (grammar.nodes[pos].sym, grammar.nodes[next].sym);
                if let Some(&(_id2, pos2)) = digrams.get(&digram) {
                    let overlaps = grammar.nodes[pos2].next == pos || grammar.nodes[pos].next == pos2;

                    assert!(overlaps);
                } else {
                    digrams.insert(digram, (id, pos));
                }
                pos = grammar.nodes[pos].next;
            }

            
        }
    }

    #[test]
    fn rule_utility(){
        let input = b"abcdabcfabababaabcde";
        let grammar = sequitur_internal(input);

        for id in 1..grammar.rules.len(){
            if grammar.rule_alive(id){
                let occurences = grammar.rules[id].occurences;
                assert!(occurences > 1);
            }
        }
    }