pub mod sequitur;
pub mod repair;
pub mod bisection;
pub mod test;

use crate::Flags;
use crate::compress::sequitur::sequitur;
use crate::compress::repair::repair;
use crate::compress::bisection::bisection;
use crate::utils::{GSize, Rule};
use huffman_coding::{HuffmanTree, HuffmanWriter};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn run(input: &PathBuf, flags: Flags) -> i8 {
    let file_content = match fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not read file '{}': {}", input.display(), e);
            return 2;
        }
    };
    let uncomp_file_metadata = match fs::metadata(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not read file '{}': {}", input.display(), e);
            return 2;
        }
    };

    let mut path = input.to_path_buf();

    path.set_file_name(format!(
        "{}.gps",
        path.file_name().unwrap().to_string_lossy()
    ));

    if path.exists() && !flags.force {
        qprintln!(
            flags.quiet,
            "File {} already exists, use force to overwrite",
            path.display()
        );
        return 2;
    }

    let grammar:Vec<Rule>;
    let start_sequence:Vec<GSize>;

    if flags.sequitur{
        (grammar, start_sequence) = sequitur(&file_content);
    }else if flags.bisection{
        (grammar, start_sequence) = bisection(&file_content);
    }else {
        (grammar, start_sequence) = repair(&file_content);
    }
    save_grammar(&grammar, &start_sequence, &path, uncomp_file_metadata.len(), flags);

    if flags.verbose {
        let comp_file_metadata = match fs::metadata(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Could not read file '{}': {}", input.display(), e);
                return 2;
            }
        };

        let comp_size = comp_file_metadata.len() as f64;
        let uncomp_size = uncomp_file_metadata.len() as f64;
        let comp_percentage = ((uncomp_size - comp_size) / uncomp_size) * 100.0;
        let comp_rate = comp_size / uncomp_size;
        let bits_per_byte = (comp_size * 8.0) / uncomp_size;

        println!(
            "input: {}, size in: {} byte, size out: {} byte, compression rate: {:.2}:1, {:.2} bits/byte, storage saved: {:.2}%",
            input.display(),
            uncomp_size,
            comp_size,
            comp_rate,
            bits_per_byte,
            comp_percentage
        );
    }
    return 0;
}

pub fn save_grammar(grammar: &Vec<Rule>, start_rule: &Vec<GSize>, path: &PathBuf, input_size: u64, flags: Flags) {
    let mut buffer: Vec<u8> = Vec::new();
    let mut in_bytes: Vec<u8> = Vec::new();

    for rule in grammar {
        for symbol in rule.expansion {
            in_bytes.extend_from_slice(&symbol.to_le_bytes());
        }
    }

    for &symbol in start_rule {
        in_bytes.extend_from_slice(&symbol.to_le_bytes());
    }

    if flags.no_huffman {
        buffer.extend_from_slice(b"GPS");
        buffer.extend_from_slice(&input_size.to_le_bytes());

        let rule_count = grammar.len() as GSize + 1;
        buffer.extend_from_slice(&rule_count.to_le_bytes());

        let uncompressed_len = in_bytes.len() as GSize;
        buffer.extend_from_slice(&uncompressed_len.to_le_bytes());
        buffer.extend_from_slice(&in_bytes);
    }else {
        let tree = HuffmanTree::from_data(&in_bytes);

        let mut encoded: Vec<u8> = Vec::new();
        {
            let mut writer = HuffmanWriter::new(&mut encoded, &tree);
            writer
                .write_all(&in_bytes)
                .expect("Huffman encoding failed!");
        }

        buffer.extend_from_slice(b"GPS");
        buffer.extend_from_slice(&input_size.to_le_bytes());

        let rule_count = grammar.len() as GSize + 1;
        buffer.extend_from_slice(&rule_count.to_le_bytes());

        let table = tree.to_table();
        buffer.extend_from_slice(&table);

        let uncompressed_len = in_bytes.len() as GSize;
        buffer.extend_from_slice(&uncompressed_len.to_le_bytes());

        buffer.extend_from_slice(&encoded);
    }


    if let Err(e) = fs::write(path, &buffer) {
        eprintln!("Could not write file '{}': {}", path.display(), e);
    }
}

pub fn _print_grammar(grammar: &Vec<Rule>) {
    for (i, rule) in grammar.iter().enumerate() {
        print!("R{} -> ", i + 256);

        for &symbol in &rule.expansion {
            if symbol == GSize::MAX {
                print!("_ ");
            } else if symbol < 256 {
                print!("'{}' ", symbol as u8 as char);
            } else {
                print!("R{} ", (symbol) as usize);
            }
        }

        println!();
    }
}

fn _build_grammar(input: &[u8], rules: &mut Vec<Rule>) -> GSize {
    if input.len() == 1 {
        let sym = input[0] as GSize;
        let idx = rules.len() as GSize + 256;
        rules.push(Rule {
            expansion: [sym, GSize::MAX],
        });
        return idx;
    }

    let mid = input.len() / 2;
    let left_idx = _build_grammar(&input[..mid], rules);
    let right_idx = _build_grammar(&input[mid..], rules);

    let idx = rules.len() as GSize + 256;
    rules.push(Rule {
        expansion: [left_idx, right_idx],
    });
    idx
}

fn _recursive_dummy_grammar(file_content: &[u8]) -> Vec<Rule> {
    let mut grammar = Vec::new();
    let _start_symbol = _build_grammar(file_content, &mut grammar);
    grammar
}

#[cfg(test)]
mod tests {
    use crate::decompress;

    use super::*;
    use std::{fs::read, io::Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_is_compressed() {
        let mut file = NamedTempFile::new().unwrap();
        let content = b"Testing".to_vec();
        file.write_all(&content).unwrap();
        let path = file.path();
        let flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,
            no_huffman: false,
        };
        let exit_code = run(&path.to_path_buf(), flags);

        let compressed_path = file.path().with_extension("gps");
        let content = fs::read(compressed_path).unwrap();

        let num_bytes: [u8; 8] = content[3..11].try_into().unwrap();
        let num = u64::from_le_bytes(num_bytes);

        assert!(content.starts_with(b"GPS"));
        assert!(num > 0);
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_force_flag() {
        let mut file = NamedTempFile::new().unwrap();
        let content = b"Testing".to_vec();
        file.write_all(&content).unwrap();
        let path = file.path().to_path_buf();

        fs::write(&path, b"Wrong Content").unwrap();
        let flags = Flags {
            quiet: false,
            verbose: false,
            force: true,
            bisection: false,
            sequitur: false,
            no_huffman: false,
        };
        let exit_code = run(&path, flags);

        let compressed_path = file.path().with_extension("gps");
        let content = fs::read(compressed_path).unwrap();

        assert!(content.starts_with(b"GPS"));
        assert_ne!(String::from_utf8_lossy(&content[11..]), "Testing");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_failed_to_overwrite() {
        let mut file = NamedTempFile::new().unwrap();
        let content = b"Testing".to_vec();
        file.write_all(&content).unwrap();
        let path = file.path().to_path_buf();
        let flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,
            no_huffman: false,
        };
        run(&path, flags);

        fs::write(&path, b"Wrong Content").unwrap();
        let flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,
            no_huffman: false,
        };
        let exit_code = run(&path, flags);

        assert_eq!(exit_code, 2);
    }

    #[test]
    fn test_compression_decompression() {
        let mut file = NamedTempFile::new().unwrap();
        let content = b"\xD6bc\xD6bcbc".to_vec();
        file.write_all(&content).unwrap();
        let path = file.path().to_path_buf();
        let mut flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,
            no_huffman: false,
        };
        run(&path, flags);

        let mut output_path = path.clone();
        output_path.set_extension("gps");

        assert!(output_path.exists(), "Failed to create compressed file");
        flags.force = true;

        let exit_code = decompress::run(&output_path.to_path_buf(), flags);

        assert_eq!(exit_code, 0, "Failed to decompress file");
        let decompressed_file_content = read(path).unwrap();
        assert_eq!(
            decompressed_file_content, content,
            "original input and decompress output are different"
        );
    }
}
