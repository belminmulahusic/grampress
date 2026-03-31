use super::*;
    use crate::compress::test;

    #[test]
    fn grammar_correct() {
        test::common::grammar_correct(repair);
    }

    #[test]
    fn grammar_correct_large_input() {
        test::common::grammar_correct_large_input(repair);
    }

    #[test]
    fn input_len_0() {
        test::common::input_len_0(repair);
    }

    #[test]
    fn input_len_1() {
        test::common::input_len_1(repair);
    }

    #[test]
    fn nonterminals_in_bounds() {
        test::common::nonterminals_in_bounds(repair);
    }