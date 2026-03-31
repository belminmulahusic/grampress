use huffman_coding::{HuffmanReader, HuffmanTree};
use std::io::Read;

pub type GSize = u32;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]

pub struct Rule {
    pub expansion: [GSize; 2],
}

pub fn convert_to_cnf(grammar: &mut Vec<Rule>, start_sequence: &Vec<GSize>) {
    let mut next_symbol: GSize = 256 + grammar.len() as GSize;

    if start_sequence.len() == 2 {
        grammar.push(Rule {
            expansion: [start_sequence[0], start_sequence[1]],
        });
        return;
    }

    let mut left = start_sequence[0];
    for &right in &start_sequence[1..] {
        let rule = Rule {
            expansion: [left, right],
        };
        grammar.push(rule);
        left = next_symbol;
        next_symbol += 1;
    }
}

pub fn load_huffman_grammar(data: &[u8]) -> (Vec<Rule>, Vec<GSize>) {
    let rule_count = GSize::from_le_bytes(data[11..15].try_into().unwrap()) as usize;
    let table: [u8; 256] = data[15..271].try_into().unwrap();
    let uncompressed_len = GSize::from_le_bytes(data[271..275].try_into().unwrap()) as usize;

    let tree = HuffmanTree::from_table(&table);

    let mut decoded = Vec::with_capacity(uncompressed_len);
    HuffmanReader::new(&data[275..], tree)
        .take(uncompressed_len as u64)
        .read_to_end(&mut decoded)
        .expect("Huffman decoding failed!");

    let mut symbols: Vec<GSize> = Vec::new();

    for i in (0..decoded.len()).step_by(4) {
        let value =
            GSize::from_le_bytes([decoded[i], decoded[i + 1], decoded[i + 2], decoded[i + 3]]);
        symbols.push(value);
    }

    let grammar_symbols = (rule_count - 1) * 2;

    let mut grammar: Vec<Rule> = Vec::new();

    for i in 0..rule_count - 1 {
        let rule = Rule {
            expansion: [symbols[i * 2], symbols[i * 2 + 1]],
        };
        grammar.push(rule);
    }

    let start_rule = symbols[grammar_symbols..].to_vec();

    (grammar, start_rule)
}

pub fn load_grammar(data: &[u8]) -> (Vec<Rule>, Vec<GSize>) {
    let rule_count = GSize::from_le_bytes(data[11..15].try_into().unwrap()) as usize;
    let uncompressed_len = GSize::from_le_bytes(data[15..19].try_into().unwrap()) as usize;
    let symbols_end = 19 + uncompressed_len;
    let symbol_bytes = &data[19..symbols_end];
    let mut symbols: Vec<GSize> = Vec::with_capacity(symbol_bytes.len() / 4);

    for chunk in symbol_bytes.chunks_exact(4) {
        let value = GSize::from_le_bytes(chunk.try_into().unwrap());
        symbols.push(value);
    }

    let grammar_symbols = (rule_count - 1) * 2;

    let mut grammar: Vec<Rule> = Vec::with_capacity(rule_count - 1);

    for i in 0..(rule_count - 1) {
        let rule = Rule {
            expansion: [symbols[i * 2], symbols[i * 2 + 1]],
        };
        grammar.push(rule);
    }

    let start_rule = symbols[grammar_symbols..].to_vec();

    (grammar, start_rule)
}
