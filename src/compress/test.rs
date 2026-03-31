  #[cfg(test)]
  
  pub mod common {
    use crate::utils::{Rule, GSize};
    use crate::decompress::derive_word;

    type CompressionAlgorithm = fn (&[u8]) -> (Vec<Rule>, Vec<GSize>);

    pub fn grammar_correct(algorithm: CompressionAlgorithm) {
        let test_inputs: Vec<&[u8]> = vec![
            b"ab",
            b"abc",
            b"abab",
            b"aaa",
            b"aaaa",
            b"abcdabcd",
            b"abcdefgh",
        ];
        for &input in test_inputs.iter() {
            let (grammar, start_rule) = algorithm(input);
            let output = derive_word(&grammar, start_rule);
            assert_eq!(input, output);
        }
    }

    pub fn grammar_correct_large_input(algorithm: CompressionAlgorithm) {
        let mut input = Vec::new();
        for i in 0..256{
            input.extend_from_slice(b"acbdabcdefg");

            if i%2==0{
                input.extend_from_slice(b"gaxctfg");
            }
            let (grammar, start_rule) = algorithm(&input);
            let output = derive_word(&grammar, start_rule);
            assert_eq!(input, output);
        }
    }

    pub fn input_len_0(algorithm: CompressionAlgorithm) {
        let input = b"";
        let (grammar, start_rule) = algorithm(input);
        assert_eq!(grammar.len(), 0);
        assert_eq!(start_rule.len(), 0);
    }

    pub fn input_len_1(algorithm: CompressionAlgorithm){
        let input = b"a";
        let (grammar, start_rule) = algorithm(input);
        assert!(grammar.is_empty());
        assert_eq!(start_rule.len(), 1);
        assert_eq!(start_rule[0] as u8, b'a');
    }

    pub fn nonterminals_in_bounds(algorithm: CompressionAlgorithm){
        let input = b"abcdabcd";        
        let (grammar, start_rule) = algorithm(input);
        let max_nonterminal = 256 + grammar.len() as u32;

        for s in start_rule{
            if s >= 256 {
                assert!(s < max_nonterminal);
            }
        }

        for rule in grammar {
            for symbol in rule.expansion{
                assert!(symbol < max_nonterminal);
            }
        }
    }
}