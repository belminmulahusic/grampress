use super::*;
use crate::compress;
use crate::utils::{GSize, Rule};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use tempfile::NamedTempFile;

const ALL_FLAGS_FALSE: Flags = Flags {
    quiet: false,
    verbose: false,
    force: false,
    bisection: false,
    sequitur: false,
    no_huffman: false,
};

// Function to create grammars
fn create_file(content: &str, flags: Flags) -> PathBuf {
    let mut input_file = NamedTempFile::new().unwrap();
    writeln!(input_file, "{}", content).unwrap();
    let input_path = input_file.path().to_path_buf();
    compress::run(&input_path, flags);

    let mut gps_path = input_path.clone();
    gps_path.set_file_name(format!(
        "{}.gps",
        input_path.file_name().unwrap().to_string_lossy()
    ));
    gps_path
}

fn remove_file_if_exists(path: &PathBuf) {
    if path.exists() {
        fs::remove_file(path).unwrap();
    }
}

pub struct GrammarBuilder {
    grammar: Vec<Rule>,
    start_sequence: Vec<GSize>,
}

impl GrammarBuilder {
    pub fn new() -> Self {
        GrammarBuilder {
            grammar: Vec::new(),
            start_sequence: Vec::new(),
        }
    }

    pub fn next_symbol(&self) -> GSize {
        256 + self.grammar.len() as GSize
    }

    pub fn push_rule(&mut self, left: GSize, right: GSize) -> GSize {
        let idx = self.next_symbol();
        self.grammar.push(Rule {
            expansion: [left, right],
        });
        idx
    }

    pub fn push_start(&mut self, symbol: GSize) {
        self.start_sequence.push(symbol);
    }

    pub fn build(self) -> (Vec<Rule>, Vec<GSize>) {
        (self.grammar, self.start_sequence)
    }
}

#[test]
fn test_file_not_exist() {
    let flags = Flags {
        quiet: false,
        verbose: true,
        force: false,
        bisection: false,
        sequitur: false,
        no_huffman: false,
    };
    let path = Path::new("wrong_path.gps");
    assert_eq!(run(path, "pattern", flags), 2);
}

#[test]
fn test_gps_header_wrong() {
    let mut file = NamedTempFile::new().unwrap();
    let header = b"ABC";
    file.write_all(header).unwrap();
    let path = file.path();
    let flags = Flags {
        quiet: true,
        verbose: false,
        force: false,
        bisection: false,
        sequitur: false,
        no_huffman: false,
    };
    let result = run(path, "random", flags);
    assert_eq!(result, 2);
}

#[test]
fn test_gps_header_correct() {
    let gps_path = create_file("Hello World!", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "Hello World!", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

// Tests for search coverage: all possible cases
#[test]
fn coverage_pattern_not_found() {
    let gps_path = create_file("Hello World!", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "xyz", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_found() {
    let gps_path = create_file("Hello World!", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "Hello World!", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_appears_multiple_times() {
    let gps_path = create_file(
        "Hello World, Hello You, Hello Universe, Hello Mars!",
        ALL_FLAGS_FALSE,
    );
    let result = run(&gps_path, "Hello", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_at_the_beginning() {
    let gps_path = create_file("Hello World!", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "He", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_in_the_middle() {
    let gps_path = create_file("Hello World!", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "lo ", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_at_the_end() {
    let gps_path = create_file("Hello World!", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "ld!", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_overlapping() {
    let gps_path = create_file("aaaaaaaaaaa", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "aaa", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_empty_pattern_empty() {
    let gps_path = create_file("", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "", ALL_FLAGS_FALSE);
    assert_eq!(result, 2);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_empty() {
    let gps_path = create_file("abcdefg", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "", ALL_FLAGS_FALSE);
    assert_eq!(result, 2);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_empty_pattern_not_empty() {
    let gps_path = create_file("", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abcdef", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_one_character_pattern_longer() {
    let gps_path = create_file("a", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abcdef", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_one_character_pattern_too() {
    let gps_path = create_file("a", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "a", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_much_shorter_than_pattern() {
    let gps_path = create_file("abcd", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abcdefghijklmnopqrstuvwxylz", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_same_length_as_pattern() {
    let gps_path = create_file("abcdefg", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abcdefg", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_long_text_pattern_late() {
    let mut long_text = "abcdefghijklmnop".repeat(1000);
    long_text.push_str("Hello");
    let text_end = "abcdefghijklmnop".repeat(10);
    long_text.push_str(&text_end);
    let gps_path = create_file(&long_text, ALL_FLAGS_FALSE);
    let result = run(&gps_path, "Hello", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_lower_pattern_upper() {
    let gps_path = create_file("abcdefg", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "ABCDEFG", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_upper_pattern_lower() {
    let gps_path = create_file("ABCDEFG", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abcdefg", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_mixed_case_pattern_partial_match() {
    let gps_path = create_file("aBcdeFg", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "AbcdeFG", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_utf8_symbols() {
    let gps_path = create_file("äöüßéñçλд中😊", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "çλд", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_at_start_and_end() {
    let gps_path = create_file("abc...xyzabc", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abc", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_multiple_identical_expansions() {
    let gps_path = create_file("xyzxyzxyz", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "xyzxyzxyz", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_prefix_suffix_overlap() {
    let gps_path = create_file("aaabaaab", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "aab", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_utf8_multibyte_pattern_partial_overlap() {
    let gps_path = create_file("äöäöäö", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "öäö", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_irregular_repetitions() {
    let gps_path = create_file("abc...abc.....abc..abc", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abc", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_partial_prefix_match_then_fail() {
    let gps_path = create_file("abc", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abcd", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_with_leading_trailing_spaces() {
    let gps_path = create_file("  hello  ", ALL_FLAGS_FALSE);
    let result = run(&gps_path, " hello ", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_with_nullbyte() {
    let gps_path = create_file("abc\0def\0ghi", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "\0d", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_almost_matches() {
    let gps_path = create_file("abcdefgh", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abcdefgi", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_with_newlines_pattern_spans_lines() {
    let gps_path = create_file("hello\nworld\ntest", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "world\ntest", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_with_tabs() {
    let gps_path = create_file("hello\tworld\ttest", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "\tworld\t", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_single_punctuation() {
    let gps_path = create_file("Hello, World!", ALL_FLAGS_FALSE);
    let result = run(&gps_path, ",", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_alternating_chars() {
    let gps_path = create_file("ababababab", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "baba", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_at_every_position() {
    let gps_path = create_file("xaxbxcxdxexfx", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "x", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_binary_like_data() {
    let gps_path = create_file("\x00\x01\x02\x03\x04\x05", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "\x02\x03\x04", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_very_long_repeating_text() {
    let long_text = "repeat".repeat(10000);
    let gps_path = create_file(&long_text, ALL_FLAGS_FALSE);
    let result = run(&gps_path, "repeatrepeat", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_at_grammar_leaf() {
    let gps_path = create_file("a", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "b", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_palindrome_text_and_pattern() {
    let gps_path = create_file("abccba", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "bccb", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_numbers_as_text() {
    let gps_path = create_file("1234567890", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "456", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_mixed_alphanumeric() {
    let gps_path = create_file("abc123def456ghi789", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "def456", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_emoji_pattern() {
    let gps_path = create_file("Hello 😊 World 🌍!", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "😊 World", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_right_to_left_text() {
    let gps_path = create_file("مرحبا العالم", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "العالم", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_chinese_characters() {
    let gps_path = create_file("你好世界", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "世界", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_twice_consecutively() {
    let gps_path = create_file("abcabcdef", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abcabc", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_pattern_every_other_char() {
    let gps_path = create_file("abcdef", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "ace", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_special_regex_chars_literal() {
    let gps_path = create_file(".*+?[]{}()|\\^$", ALL_FLAGS_FALSE);
    let result = run(&gps_path, ".*+?", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_carriage_return_line_feed() {
    let gps_path = create_file("line1\r\nline2\r\nline3", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "\r\nline2\r\n", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_only_whitespace_text() {
    let gps_path = create_file("     ", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "  ", ALL_FLAGS_FALSE);
    assert_eq!(result, 0);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_ends_with_pattern_start() {
    let gps_path = create_file("prefixab", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abc", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn coverage_text_starts_with_pattern_end() {
    let gps_path = create_file("cdsuffix", ALL_FLAGS_FALSE);
    let result = run(&gps_path, "abcd", ALL_FLAGS_FALSE);
    assert_eq!(result, 1);
    remove_file_if_exists(&gps_path);
}

#[test]
fn index_search_pattern_expands_over_multiple_rules() {
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize);
    builder.push_rule('x' as GSize, 256);
    builder.push_rule(257, 'y' as GSize);
    builder.push_start(258);

    let (grammar, start) = builder.build();
    assert!(match_pattern(&grammar, &start, b"xaby").1);
    assert!(!match_pattern(&grammar, &start, b"xy").1);
}

#[test]
fn pattern_entirely_in_left_child() {
    // Grammar: R256 -> "abc", R257 -> "xyz", R258 -> R256 R257 = "abcxyz"
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule(256, 'c' as GSize); // R257 -> "abc"
    builder.push_rule('x' as GSize, 'y' as GSize); // R258 -> "xy"
    builder.push_rule(258, 'z' as GSize); // R259 -> "xyz"
    builder.push_rule(257, 259); // R260 -> "abcxyz"
    builder.push_start(260);

    let (grammar, start) = builder.build();
    assert!(match_pattern(&grammar, &start, b"ab").1, "left child: ab");
    assert!(match_pattern(&grammar, &start, b"abc").1, "left child: abc");
    assert!(match_pattern(&grammar, &start, b"bc").1, "left child: bc");
}

#[test]
fn pattern_entirely_in_right_child() {
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule(256, 'c' as GSize); // R257 -> "abc"
    builder.push_rule('x' as GSize, 'y' as GSize); // R258 -> "xy"
    builder.push_rule(258, 'z' as GSize); // R259 -> "xyz"
    builder.push_rule(257, 259); // R260 -> "abcxyz"
    builder.push_start(260);

    let (grammar, start) = builder.build();
    assert!(match_pattern(&grammar, &start, b"xy").1, "right child: xy");
    assert!(match_pattern(&grammar, &start, b"xyz").1, "right child: xyz");
    assert!(match_pattern(&grammar, &start, b"yz").1, "right child: yz");
}

#[test]
fn pattern_spans_boundary_small_overlap() {
    // Pattern crosses boundary with minimal overlap on each side
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule('c' as GSize, 'd' as GSize); // R257 -> "cd"
    builder.push_rule(256, 257); // R258 -> "abcd"
    builder.push_start(258);

    let (grammar, start) = builder.build();

    assert!(
        match_pattern(&grammar, &start, b"bc").1,
        "boundary: 1 char each side"
    );
}

#[test]
fn pattern_spans_boundary_left_heavy() {
    // Pattern mostly in left, small part in right
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule(256, 'c' as GSize); // R257 -> "abc"
    builder.push_rule('x' as GSize, 'y' as GSize); // R258 -> "xy"
    builder.push_rule(257, 258); // R259 -> "abcxy"
    builder.push_start(259);

    let (grammar, start) = builder.build();

    assert!(
        match_pattern(&grammar, &start, b"bcx").1,
        "boundary left-heavy: bcx"
    );
    assert!(
        match_pattern(&grammar, &start, b"abcx").1,
        "boundary left-heavy: abcx"
    );
}

#[test]
fn pattern_spans_boundary_right_heavy() {
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule('x' as GSize, 'y' as GSize); // R257 -> "xy"
    builder.push_rule(257, 'z' as GSize); // R258 -> R257 'z' = "xyz"
    builder.push_rule(256, 258); // R259 -> R256 R258 = "abxyz"
    builder.push_start(259);

    let (grammar, start) = builder.build();

    assert!(
        match_pattern(&grammar, &start, b"bxy").1,
        "boundary right-heavy: bxy"
    );
    assert!(
        match_pattern(&grammar, &start, b"bxyz").1,
        "boundary right-heavy: bxyz"
    );
}

#[test]
fn pattern_spans_entire_expansion() {
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule('c' as GSize, 'd' as GSize); // R257 -> "cd"
    builder.push_rule(256, 257); // R258 -> "abcd"
    builder.push_start(258);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"abcd").1, "entire expansion");
}

#[test]
fn pattern_spans_deeply_nested_boundary() {
    // Deep tree: ((a,b),(c,d)) -> "abcd"
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule('c' as GSize, 'd' as GSize); // R257 -> "cd"
    builder.push_rule(256, 257); // R258 -> "abcd"
    builder.push_start(258);

    let (grammar, start) = builder.build();

    // Pattern spans inner boundary of left child
    assert!(match_pattern(&grammar, &start, b"ab").1);
    // Pattern spans inner boundary of right child
    assert!(match_pattern(&grammar, &start, b"cd").1);
    // Pattern spans outer boundary
    assert!(match_pattern(&grammar, &start, b"bc").1);
}

#[test]
fn pattern_spans_three_level_deep_boundary() {
    // (((a,b),c),(d,(e,f))) -> "abcdef"
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule(256, 'c' as GSize); // R257 -> "abc"
    builder.push_rule('e' as GSize, 'f' as GSize); // R258 -> "ef"
    builder.push_rule('d' as GSize, 258); // R259 -> "def"
    builder.push_rule(257, 259); // R260 -> "abcdef"
    builder.push_start(260);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"cd").1, "deep boundary: cd");
    assert!(match_pattern(&grammar, &start, b"bcde").1, "deep boundary: bcde");
    assert!(
        match_pattern(&grammar, &start, b"cdef").1,
        "spans into right subtree"
    );
}

#[test]
fn pattern_with_repeated_rule() {
    // R256 -> "ab", R257 -> R256 R256 = "abab"
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule(256, 256); // R257 -> "abab"
    builder.push_start(257);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"abab").1, "full repeated");
    assert!(match_pattern(&grammar, &start, b"ba").1, "boundary of repeated: ba");
    assert!(
        match_pattern(&grammar, &start, b"bab").1,
        "overlapping repeated: bab"
    );
}

#[test]
fn pattern_with_triple_repetition() {
    // "ababab" via R256 -> "ab", R257 -> R256 R256, R258 -> R257 R256
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule(256, 256); // R257 -> "abab"
    builder.push_rule(257, 256); // R258 -> "ababab"
    builder.push_start(258);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"ababab").1, "triple");
    assert!(match_pattern(&grammar, &start, b"babab").1, "offset in triple");
    assert!(match_pattern(&grammar, &start, b"baba").1, "middle of triple");
}

#[test]
fn pattern_single_character_terminals() {
    let mut builder = GrammarBuilder::new();
    builder.push_rule('x' as GSize, 'y' as GSize);
    builder.push_start(256);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"x").1, "single x");
    assert!(match_pattern(&grammar, &start, b"y").1, "single y");
    assert!(!match_pattern(&grammar, &start, b"z").1, "single z not present");
}

#[test]
fn pattern_single_character_at_boundary() {
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule('c' as GSize, 'd' as GSize); // R257 -> "cd"
    builder.push_rule(256, 257); // R258 -> "abcd"
    builder.push_start(258);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"b").1, "last char of left");
    assert!(match_pattern(&grammar, &start, b"c").1, "first char of right");
}

#[test]
fn pattern_longer_than_expansion() {
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // "ab"
    builder.push_start(256);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"abc").1, "too long");
    assert!(!match_pattern(&grammar, &start, b"abab").1, "way too long");
}

#[test]
fn pattern_not_in_grammar() {
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize);
    builder.push_rule(256, 'c' as GSize); // "abc"
    builder.push_start(257);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"xyz").1, "completely different");
    assert!(!match_pattern(&grammar, &start, b"abd").1, "one char wrong");
    assert!(!match_pattern(&grammar, &start, b"ac").1, "skipped char");
}

#[test]
fn pattern_partial_match_but_not_contiguous() {
    // Grammar expands to "abcd", pattern "ad" exists as chars but not contiguous
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize);
    builder.push_rule('c' as GSize, 'd' as GSize);
    builder.push_rule(256, 257); // "abcd"
    builder.push_start(258);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"ad").1);
    assert!(!match_pattern(&grammar, &start, b"ac").1);
    assert!(!match_pattern(&grammar, &start, b"bd").1);
}

#[test]
fn pattern_with_special_bytes() {
    let mut builder = GrammarBuilder::new();
    builder.push_rule(0, 255); // R256 -> \x00 \xff
    builder.push_rule(256, '\n' as GSize); // R257 -> \x00\xff\n
    builder.push_start(257);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, &[0, 255]).1, "null and 0xff");
    assert!(
        match_pattern(&grammar, &start, &[255, b'\n']).1,
        "boundary with newline"
    );
    assert!(
        match_pattern(&grammar, &start, &[0, 255, b'\n']).1,
        "full special"
    );
}

#[test]
fn pattern_with_repeated_bytes() {
    // "aaaa"
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'a' as GSize); // R256 -> "aa"
    builder.push_rule(256, 256); // R257 -> "aaaa"
    builder.push_start(257);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"a").1, "single a");
    assert!(match_pattern(&grammar, &start, b"aa").1, "double a");
    assert!(match_pattern(&grammar, &start, b"aaa").1, "triple a");
    assert!(match_pattern(&grammar, &start, b"aaaa").1, "quad a");
    assert!(!match_pattern(&grammar, &start, b"aaaaa").1, "too many a's");
}

#[test]
fn pattern_in_unbalanced_tree_left_heavy() {
    // Left-heavy: (((a,b),c),d) -> "abcd"
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule(256, 'c' as GSize); // R257 -> "abc"
    builder.push_rule(257, 'd' as GSize); // R258 -> "abcd"
    builder.push_start(258);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"ab").1);
    assert!(match_pattern(&grammar, &start, b"bc").1);
    assert!(match_pattern(&grammar, &start, b"cd").1);
    assert!(match_pattern(&grammar, &start, b"abcd").1);
}

#[test]
fn pattern_in_unbalanced_tree_right_heavy() {
    // Right-heavy: (a,(b,(c,d))) -> "abcd"
    let mut builder = GrammarBuilder::new();
    builder.push_rule('c' as GSize, 'd' as GSize); // R256 -> "cd"
    builder.push_rule('b' as GSize, 256); // R257 -> "bcd"
    builder.push_rule('a' as GSize, 257); // R258 -> "abcd"
    builder.push_start(258);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"ab").1);
    assert!(match_pattern(&grammar, &start, b"bc").1);
    assert!(match_pattern(&grammar, &start, b"cd").1);
    assert!(match_pattern(&grammar, &start, b"abcd").1);
}

#[test]
fn pattern_spans_multiple_boundaries_simultaneously() {
    // ((a,b),(c,d)) where pattern "bcd" spans left->right AND into right's children
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule('c' as GSize, 'd' as GSize); // R257 -> "cd"
    builder.push_rule(256, 257); // R258 -> "abcd"
    builder.push_start(258);

    let (grammar, start) = builder.build();

    assert!(
        match_pattern(&grammar, &start, b"bcd").1,
        "spans main + right internal"
    );
    assert!(
        match_pattern(&grammar, &start, b"abc").1,
        "spans left internal + main"
    );
}

#[test]
fn pattern_in_longer_expansion() {
    // "abcdefgh"
    let mut builder = GrammarBuilder::new();
    builder.push_rule('a' as GSize, 'b' as GSize); // R256 -> "ab"
    builder.push_rule('c' as GSize, 'd' as GSize); // R257 -> "cd"
    builder.push_rule('e' as GSize, 'f' as GSize); // R258 -> "ef"
    builder.push_rule('g' as GSize, 'h' as GSize); // R259 -> "gh"
    builder.push_rule(256, 257); // R260 -> "abcd"
    builder.push_rule(258, 259); // R261 -> "efgh"
    builder.push_rule(260, 261); // R262 -> "abcdefgh"
    builder.push_start(262);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"de").1, "middle boundary");
    assert!(match_pattern(&grammar, &start, b"cdef").1, "spans center");
    assert!(match_pattern(&grammar, &start, b"bcdefg").1, "large span");
    assert!(match_pattern(&grammar, &start, b"abcdefgh").1, "full");
    assert!(!match_pattern(&grammar, &start, b"abcdefghi").1, "too long");
}

#[test]
fn false_positive_non_contiguous_chars() {
    // "abcd" - chars exist but not in sequence
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let cd = builder.push_rule('c' as GSize, 'd' as GSize);
    let abcd = builder.push_rule(ab, cd);
    builder.push_start(abcd);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"ad").1, "skip bc");
    assert!(!match_pattern(&grammar, &start, b"ac").1, "skip b");
    assert!(!match_pattern(&grammar, &start, b"bd").1, "skip c");
    assert!(!match_pattern(&grammar, &start, b"acd").1, "skip b");
    assert!(!match_pattern(&grammar, &start, b"abd").1, "skip c");
}

#[test]
fn false_positive_reversed_order() {
    // "abcd" - pattern exists but reversed
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let cd = builder.push_rule('c' as GSize, 'd' as GSize);
    let abcd = builder.push_rule(ab, cd);
    builder.push_start(abcd);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"ba").1, "reversed ab");
    assert!(!match_pattern(&grammar, &start, b"dc").1, "reversed cd");
    assert!(!match_pattern(&grammar, &start, b"dcba").1, "reversed all");
    assert!(!match_pattern(&grammar, &start, b"cb").1, "reversed boundary");
}

#[test]
fn false_positive_wrong_boundary_combination() {
    // "ab" + "xy" = "abxy", don't match mixed boundaries
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let xy = builder.push_rule('x' as GSize, 'y' as GSize);
    let abxy = builder.push_rule(ab, xy);
    builder.push_start(abxy);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"ay").1, "wrong: a + y");
    assert!(!match_pattern(&grammar, &start, b"ax").1, "wrong: a + x");
    assert!(!match_pattern(&grammar, &start, b"by").1, "wrong: b + y (skips x)");
}

#[test]
fn false_positive_similar_but_different() {
    // "abc" - similar patterns that don't exist
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let abc = builder.push_rule(ab, 'c' as GSize);
    builder.push_start(abc);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"abd").1, "wrong last char");
    assert!(!match_pattern(&grammar, &start, b"xbc").1, "wrong first char");
    assert!(!match_pattern(&grammar, &start, b"axc").1, "wrong middle char");
    assert!(!match_pattern(&grammar, &start, b"abcd").1, "too long");
    assert!(!match_pattern(&grammar, &start, b"zabc").1, "prefix added");
}


#[test]
fn false_positive_repeated_chars_wrong_count() {
    // "aaaa" - wrong number of repetitions
    let mut builder = GrammarBuilder::new();
    let aa = builder.push_rule('a' as GSize, 'a' as GSize);
    let aaaa = builder.push_rule(aa, aa);
    builder.push_start(aaaa);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"aaaaa").1, "5 a's");
    assert!(!match_pattern(&grammar, &start, b"aaaaaa").1, "6 a's");
}

#[test]
fn false_positive_overlapping_but_not_present() {
    // "abcabc" - overlapping pattern that doesn't exist
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let abc = builder.push_rule(ab, 'c' as GSize);
    let abcabc = builder.push_rule(abc, abc);
    builder.push_start(abcabc);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"abcabc").1, "full match");
    assert!(match_pattern(&grammar, &start, b"cab").1, "boundary");
    assert!(match_pattern(&grammar, &start, b"bca").1, "boundary 2");

    assert!(!match_pattern(&grammar, &start, b"aba").1, "not contiguous");
    assert!(!match_pattern(&grammar, &start, b"cac").1, "not contiguous");
    assert!(!match_pattern(&grammar, &start, b"abab").1, "not present");
    assert!(!match_pattern(&grammar, &start, b"bcbc").1, "not present");
}

#[test]
fn false_positive_deep_tree_wrong_combinations() {
    // Deep: (((a,b),c),(d,(e,f))) = "abcdef"
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let abc = builder.push_rule(ab, 'c' as GSize);
    let ef = builder.push_rule('e' as GSize, 'f' as GSize);
    let def = builder.push_rule('d' as GSize, ef);
    let abcdef = builder.push_rule(abc, def);
    builder.push_start(abcdef);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"af").1, "skip bcde");
    assert!(!match_pattern(&grammar, &start, b"ae").1, "skip bcd");
    assert!(!match_pattern(&grammar, &start, b"bf").1, "skip cde");
    assert!(!match_pattern(&grammar, &start, b"cf").1, "skip de");
    assert!(!match_pattern(&grammar, &start, b"adf").1, "skip bc, e");
    assert!(!match_pattern(&grammar, &start, b"ace").1, "skip b, d");
    assert!(!match_pattern(&grammar, &start, b"bdf").1, "skip c, e");
}

#[test]
fn false_positive_suffix_prefix_but_not_adjacent() {
    // "xaby" - test suffix/prefix combinations carefully
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let xab = builder.push_rule('x' as GSize, ab);
    let xaby = builder.push_rule(xab, 'y' as GSize);
    builder.push_start(xaby);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"xby").1, "skip a");
}

#[test]
fn false_positive_all_same_char_different_positions() {
    // "abab" - same chars at different positions
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let abab = builder.push_rule(ab, ab);
    builder.push_start(abab);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"abab").1, "full");
    assert!(match_pattern(&grammar, &start, b"aba").1, "3 chars");
    assert!(match_pattern(&grammar, &start, b"bab").1, "3 chars offset");

    assert!(!match_pattern(&grammar, &start, b"aa").1, "skip b's");
    assert!(!match_pattern(&grammar, &start, b"bb").1, "skip a's");
    assert!(!match_pattern(&grammar, &start, b"aab").1, "wrong order");
    assert!(!match_pattern(&grammar, &start, b"abb").1, "wrong order");
}

#[test]
fn false_positive_three_children_boundaries() {
    // "abc" + "def" + "ghi" via ((abc, def), ghi) = "abcdefghi"
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let abc = builder.push_rule(ab, 'c' as GSize);
    let de = builder.push_rule('d' as GSize, 'e' as GSize);
    let def = builder.push_rule(de, 'f' as GSize);
    let gh = builder.push_rule('g' as GSize, 'h' as GSize);
    let ghi = builder.push_rule(gh, 'i' as GSize);
    let abcdef = builder.push_rule(abc, def);
    let all = builder.push_rule(abcdef, ghi);
    builder.push_start(all);

    let (grammar, start) = builder.build();

    // Valid patterns
    assert!(match_pattern(&grammar, &start, b"cde").1, "first boundary");
    assert!(match_pattern(&grammar, &start, b"fgh").1, "second boundary");
    assert!(match_pattern(&grammar, &start, b"cdefgh").1, "spans both");

    // Invalid: skipping chars
    assert!(!match_pattern(&grammar, &start, b"ag").1, "skip bcdef");
    assert!(!match_pattern(&grammar, &start, b"ci").1, "skip defgh");
    assert!(!match_pattern(&grammar, &start, b"agi").1, "skip all middle");
    assert!(!match_pattern(&grammar, &start, b"beh").1, "skip c,d,f,g");
    assert!(!match_pattern(&grammar, &start, b"cfi").1, "skip de, gh");
}

#[test]
fn false_positive_binary_bytes() {
    // Test with actual byte values, not just ASCII
    let mut builder = GrammarBuilder::new();
    let r1 = builder.push_rule(0, 1);
    let r2 = builder.push_rule(2, 3);
    let r3 = builder.push_rule(r1, r2);
    builder.push_start(r3);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, &[0, 1, 2, 3]).1, "full");
    assert!(match_pattern(&grammar, &start, &[1, 2]).1, "boundary");

    assert!(!match_pattern(&grammar, &start, &[0, 2]).1, "skip 1");
    assert!(!match_pattern(&grammar, &start, &[0, 3]).1, "skip 1,2");
    assert!(!match_pattern(&grammar, &start, &[1, 3]).1, "skip 2");
}

#[test]
fn false_positive_single_char_not_present() {
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    builder.push_start(ab);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"a").1, "a present");
    assert!(match_pattern(&grammar, &start, b"b").1, "b present");

    for c in b'c'..=b'z' {
        assert!(
            !match_pattern(&grammar, &start, &[c]).1,
            "char {} should not match",
            c as char
        );
    }
}

#[test]
fn false_positive_long_pattern_short_grammar() {
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    builder.push_start(ab);

    let (grammar, start) = builder.build();

    assert!(!match_pattern(&grammar, &start, b"abc").1, "too long");
    assert!(!match_pattern(&grammar, &start, b"aab").1, "too long");
    assert!(!match_pattern(&grammar, &start, b"abb").1, "too long");
    assert!(!match_pattern(&grammar, &start, b"abab").1, "way too long");
    assert!(!match_pattern(&grammar, &start, b"abcdefghij").1, "much too long");
}

#[test]
fn false_positive_same_char_multiple_rules() {
    // Multiple rules using same char: shouldn't create phantom matches
    // "ab" + "bc" = "abbc"
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let bc = builder.push_rule('b' as GSize, 'c' as GSize);
    let abbc = builder.push_rule(ab, bc);
    builder.push_start(abbc);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"abbc").1, "full");
    assert!(match_pattern(&grammar, &start, b"abb").1, "prefix");
    assert!(match_pattern(&grammar, &start, b"bbc").1, "suffix");
    assert!(match_pattern(&grammar, &start, b"bb").1, "middle");

    assert!(!match_pattern(&grammar, &start, b"abc").1, "only one b");
    assert!(!match_pattern(&grammar, &start, b"ac").1, "skip both b's");
}

#[test]
fn false_positive_interleaved_pattern() {
    // "aabb" - check interleaved doesn't match
    let mut builder = GrammarBuilder::new();
    let aa = builder.push_rule('a' as GSize, 'a' as GSize);
    let bb = builder.push_rule('b' as GSize, 'b' as GSize);
    let aabb = builder.push_rule(aa, bb);
    builder.push_start(aabb);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"aabb").1, "full");
    assert!(match_pattern(&grammar, &start, b"ab").1, "boundary");
    assert!(match_pattern(&grammar, &start, b"aab").1, "3 chars");
    assert!(match_pattern(&grammar, &start, b"abb").1, "3 chars");

    assert!(!match_pattern(&grammar, &start, b"abab").1, "interleaved");
    assert!(!match_pattern(&grammar, &start, b"baba").1, "interleaved reversed");
    assert!(!match_pattern(&grammar, &start, b"abba").1, "palindrome");
    assert!(!match_pattern(&grammar, &start, b"baab").1, "reversed");
}

#[test]
fn mytest() {
    let mut builder = GrammarBuilder::new();
    let ba = builder.push_rule('b' as GSize, 'a' as GSize);
    let baba = builder.push_rule(ba, ba);
    let xa = builder.push_rule('x' as GSize, 'a' as GSize);
    let xababa = builder.push_rule(xa, baba);
    builder.push_start(xababa);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"abab").1, "abab");
}

#[test]
fn mytest2() {
    let mut builder = GrammarBuilder::new();
    let ba = builder.push_rule('b' as GSize, 'a' as GSize);
    let aba = builder.push_rule('a' as GSize, ba);
    let bab = builder.push_rule(ba, 'b' as GSize);
    let ababab = builder.push_rule(aba, bab);
    builder.push_start(ababab);

    let (grammar, start) = builder.build();

    assert!(match_pattern(&grammar, &start, b"abab").1, "ababab");
}

#[test]
fn count_overlapping_aaaa_pattern_aa() {
    let mut builder = GrammarBuilder::new();
    let aa = builder.push_rule('a' as GSize, 'a' as GSize);
    let aaaa = builder.push_rule(aa, aa);
    builder.push_start(aaaa);

    let (grammar, start) = builder.build();

    let (occ, found) = match_pattern(&grammar, &start, b"aa");
    assert!(found);
    assert_eq!(occ, 3);
}

#[test]
fn count_overlapping_ababa_pattern_aba() {
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let aba = builder.push_rule(ab, 'a' as GSize);
    let ba = builder.push_rule('b' as GSize, 'a' as GSize);
    let ababa = builder.push_rule(aba, ba);
    builder.push_start(ababa);

    let (grammar, start) = builder.build();

    let (occ, found) = match_pattern(&grammar, &start, b"aba");
    assert!(found);
    assert_eq!(occ, 2);
}

#[test]
fn count_two_non_overlapping_abab_pattern_ab() {
    let mut builder = GrammarBuilder::new();
    let ab = builder.push_rule('a' as GSize, 'b' as GSize);
    let abab = builder.push_rule(ab, ab);
    builder.push_start(abab);

    let (grammar, start) = builder.build();

    let (occ, found) = match_pattern(&grammar, &start, b"ab");
    assert!(found);
    assert_eq!(occ, 2);
}

#[test]
fn count_cross_boundary_bcbc_pattern_bc() {
    let mut builder = GrammarBuilder::new();
    let bc = builder.push_rule('b' as GSize, 'c' as GSize);
    let bcbc = builder.push_rule(bc, bc);
    builder.push_start(bcbc);

    let (grammar, start) = builder.build();

    let (occ, found) = match_pattern(&grammar, &start, b"bc");
    assert!(found);
    assert_eq!(occ, 2);
}
