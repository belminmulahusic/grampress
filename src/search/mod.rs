use crate::{Flags, utils};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Debug)]
struct Info {
    len: u64,
    right_set: Vec<bool>,
    left_set: Vec<bool>,
    occ: u64,
}

fn combine_infos(left_info: &Info, right_info: &Info, m: usize) -> Info {
    let n_left = left_info.len;
    let n_right = right_info.len;
    let left_r_set = &left_info.right_set;
    let left_l_set = &left_info.left_set;
    let right_r_set = &right_info.right_set;
    let right_l_set = &right_info.left_set;

    // Initialize new info object for rule A
    let n_new = n_left + n_right;
    let mut new_info = Info {
        len: n_new,
        right_set: vec![false; m],
        left_set: vec![false; m],
        occ: left_info.occ + right_info.occ,
    };

    let mut cross_count: u64 = 0;

    for i in 0..m {
        // Case 1: R set
        if right_r_set[i] && (i + 1) as u64 <= n_right {
            new_info.right_set[i] = true;
        }

        // Case 2: L set
        if i < m - 1 && left_r_set[i] && right_l_set[i + 1] {
            // Match-Check
            if (i + 1) as u64 <= n_left && (m - i - 1) as u64 <= n_right {
                cross_count += 1;
            }else {
                let j_new = i as u64 + n_right;
                if j_new < m as u64 {
                    new_info.right_set[j_new as usize] = true;
                }
            }

        }

        // L set
        if left_l_set[i] {
            let suffix_len = m - i;

            if n_left >= suffix_len as u64 {
                new_info.left_set[i] = true;
            } else if i as u64 + n_left < m as u64
                && right_l_set[(i as u64 + n_left) as usize]
            {
                new_info.left_set[i] = true;
            }
        }
    }

    new_info.occ += cross_count;
    new_info
}

fn match_pattern(
    grammar: &Vec<utils::Rule>,
    start_rule: &Vec<utils::GSize>,
    pattern_bytes: &[u8],
) -> (u64, bool) {
    if start_rule.is_empty() {
        return (0, false);
    }

    let total_symbols = 256 + grammar.len();
    let m = pattern_bytes.len();

    if m == 1 {
        let target = pattern_bytes[0];

        let mut occ = vec![0u64; total_symbols];
        for b in 0u16..256 {
            occ[b as usize] = if b as u8 == target { 1 } else { 0 };
        }

        for (rule_index, rule) in grammar.iter().enumerate() {
            let left = rule.expansion[0] as usize;
            let right = rule.expansion[1] as usize;
            occ[256 + rule_index] = occ[left] + occ[right];
        }

        let mut total = 0u64;
        for &sym in start_rule {
            total += occ[sym as usize];
        }

        return (total, total > 0);
    }

    // Count how often a child appears somewhere on the right-hand side of a rule (including the start rule)
    let mut parent_count = vec![0u32; total_symbols];
    for rule in grammar.iter() {
        let left = rule.expansion[0] as usize;
        let right = rule.expansion[1] as usize;
        parent_count[left] += 1;
        parent_count[right] += 1;
    }
    for &sym in start_rule.iter() {
        let idx = sym as usize;
        if idx < parent_count.len() {
            parent_count[idx] += 1;
        }
    }

    let mut memory: Vec<Option<Info>> = vec![None; total_symbols];

    // Iterate over all terminal values from 0 to 255
    for byte_val in 0u16..256 {
        let mut info = Info {
            len: 1,
            right_set: vec![false; m],
            left_set: vec![false; m],
            occ: 0,
        };

        for j in 1..m {
            let k = j - 1;
            if pattern_bytes[j] == byte_val as u8 {
                info.left_set[j] = true;
            }
            if pattern_bytes[k] == byte_val as u8 {
                info.right_set[k] = true;
            }
        }
        memory[byte_val as usize] = Some(info)
    }

    // Iterate over all nonterminals from 256 to grammar size
    for (rule_index, rule) in grammar.iter().enumerate() {
        let left_child = rule.expansion[0] as usize;
        let right_child = rule.expansion[1] as usize;

        let left_info = memory[left_child].as_ref().expect("Left child info missing");
        let right_info = memory[right_child].as_ref().expect("Right child info missing");
        let new_info = combine_infos(left_info, right_info, m);

        memory[256 + rule_index] = Some(new_info);

        for &child in &[left_child, right_child] {
            parent_count[child] -= 1;
            if child >= 256 && parent_count[child] == 0 {
                memory[child] = None;
            }
        }
    }

    // From here on, the start rule is converted to CNF on-the-fly and the information is computed
    let first_sym = start_rule[0] as usize;
    let mut acc = memory[first_sym].as_ref().expect("Info missing").clone();

    for &sym in &start_rule[1..] {
        let idx = sym as usize;
        let right_child = memory[idx].as_ref().expect("Info missing");

        let new_info = combine_infos(&acc, right_child, m);
        acc = new_info;
    }

    (acc.occ, acc.occ > 0)
}

pub fn run(input: &Path, pattern: &str, flags: Flags) -> i8 {
    let start = Instant::now();
    let data = match fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            if !flags.quiet {
                eprintln!("Could not read file '{}': {}", input.display(), e);
            }
            return 2;
        }
    };

    if &data[..3] != b"GPS" {
        if !flags.quiet {
            eprintln!("Not a gpress file!");
        }
        return 2;
    }

    let file_size = match data[3..11].try_into() {
        Ok(bytes) => u64::from_le_bytes(bytes),
        Err(_) => {
            if !flags.quiet {
                eprintln!("Invalid file size!");
            }
            return 2;
        }
    };

    if file_size <= 0 {
        if !flags.quiet {
            eprintln!("File size too small!");
        }
        return 2;
    }

    if pattern.is_empty() {
        if !flags.quiet {
            eprintln!("Pattern cannot be empty!");
        }
        return 2;
    }
    let (grammar, start_rule);
    if flags.no_huffman {
        (grammar, start_rule) = utils::load_grammar(&data);
    }else {
        (grammar, start_rule) = utils::load_huffman_grammar(&data);
    }
    let start_search = Instant::now();
    let (occ, pattern_found) = match_pattern(&grammar, &start_rule, pattern.as_bytes());
    let duration_search = start_search.elapsed();
    let duration = start.elapsed();
    if pattern_found {
        if !flags.quiet {
            println!("Found pattern {} times!", occ);
            if flags.verbose {
                println!("  File: {}", input.display());
                println!("  Pattern: {}", pattern);
                println!("  Pattern size in bytes: {}", pattern.as_bytes().len());
                println!("  Grammar size: {}", grammar.len());
                println!("  Startrule size: {}", start_rule.len());
                println!("  Total time taken: {:?}", duration);
                println!("  Search-only time taken: {:?}", duration_search);
                println!("  Count: {}", occ);
            }
        }
        return 0;
    } else {
        if !flags.quiet {
            println!("Not found...");
            if flags.verbose {
                println!("  File: {}", input.display());
                println!("  Pattern: {}", pattern);
                println!("  Pattern size in bytes: {}", pattern.as_bytes().len());
                println!("  Grammar size: {}", grammar.len());
                println!("  Startrule size: {}", start_rule.len());
                println!("  Total time taken: {:?}", duration);
                println!("  Search-only time taken: {:?}", duration_search);
                println!("  Count: {}", occ);
            }
        }
        return 1;
    }
}

#[cfg(test)]
mod tests;
