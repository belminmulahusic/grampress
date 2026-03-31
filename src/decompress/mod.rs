use crate::Flags;
use crate::utils::{GSize, Rule, convert_to_cnf, load_huffman_grammar};
use std::fs;
use std::path::Path;

pub fn run(input: &Path, flags: Flags) -> i8 {
    let comp_file_content = match fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not read file '{}': {}", input.display(), e);
            return 2;
        }
    };

    if comp_file_content.len() < 273 || &comp_file_content[..3] != b"GPS" {
        if !flags.quiet {
            eprintln!("Not a gpress file!");
        }
        return 2;
    }

    let comp_file_metadata = match fs::metadata(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not read file '{}': {}", input.display(), e);
            return 2;
        }
    };

    let (mut grammar, start_rule) = load_huffman_grammar(&comp_file_content);
    convert_to_cnf(&mut grammar, &start_rule);
    let word = derive_word(&grammar, start_rule);
    let mut path = input.to_path_buf();

    path.set_extension("");

    if path.exists() && !flags.force {
        qprintln!(
            flags.quiet,
            "File {} already exists, use force to overwrite",
            path.display()
        );
        return 2;
    } else {
        fs::write(&path, &word).expect("Failed to write compressed file");
    }

    if flags.verbose {
        let uncomp_file_metadata = match fs::metadata(&path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Could not read file '{}': {}", input.display(), e);
                return 2;
            }
        };

        let comp_size = comp_file_metadata.len() as f64;
        let uncomp_size = uncomp_file_metadata.len() as f64;

        println!(
            "input: {}, size in: {} byte, size out: {} byte",
            input.display(),
            comp_size,
            uncomp_size
        );
    }
    return 0;
}

fn _print_grammar(grammar: &Vec<Rule>) {
    for (i, rule) in grammar.iter().enumerate() {
        print!("R{} -> ", i);

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

pub fn list(input: &Path) -> i8 {
    let comp_file_content = match fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not read file '{}': {}", input.display(), e);
            return 2;
        }
    };
    if comp_file_content.len() < 11 || &comp_file_content[..3] != b"GPS" {
        eprintln!("Not a gpress file!");
        return 2;
    }

    let file_size = match comp_file_content[3..11].try_into() {
        Ok(bytes) => u64::from_le_bytes(bytes),
        Err(_) => {
            eprintln!("Invalid file size!");
            return 2;
        }
    };

    if file_size <= 0 {
        eprintln!("File size too small!");
        return 2;
    }

    let comp_file_metadata = match fs::metadata(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Could not read file '{}': {}", input.display(), e);
            return 2;
        }
    };

    let size = &comp_file_content[3..11];
    let uncomp_size = u64::from_le_bytes(size.try_into().unwrap());

    println!(
        "Uncompressed size: {:.0}, Compressed size: {:.0}, Compression ratio: {}%",
        uncomp_size,
        comp_file_metadata.len(),
        0
    );
    return 0;
}

pub fn derive_word(grammar: &Vec<Rule>, start_rule: Vec<GSize>) -> Vec<u8> {
    let mut result = Vec::new();

    for symbol in start_rule {
        expand_symbol(grammar, symbol, &mut result);
    }
    result
}

fn expand_symbol(grammar: &Vec<Rule>, symbol: GSize, output: &mut Vec<u8>) {
    if symbol < 256 {
        output.push(symbol as u8);
    } else {
        let rule_index = (symbol - 256) as usize;

        if rule_index >= grammar.len() {
            eprintln!("expand_rule: invalid rule_index {}", rule_index);
            return;
        }

        let rule = &grammar[rule_index];

        for &symbol in &rule.expansion {
            expand_symbol(grammar, symbol, output);
        }
    }
}
#[cfg(test)]
mod tests;

