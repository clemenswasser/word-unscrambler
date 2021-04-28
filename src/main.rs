use std::collections::HashMap;

use itertools::Itertools;

fn compute_string_hash(s: &str) -> u64 {
    s.bytes().map(|c| c as u64).fold(0, u64::wrapping_add)
}

fn find_word_candidates(
    dictionary_map: &HashMap<u64, Vec<String>>,
    search_word: &str,
    first_char: Option<char>,
) -> Vec<String> {
    dictionary_map
        .get(&compute_string_hash(search_word))
        .map_or_else(Vec::new, |words| {
            words
                .iter()
                .filter(|word| {
                    first_char.map_or(true, |first_char| {
                        word.chars()
                            .next()
                            .map_or(true, |first| first.to_ascii_uppercase() == first_char)
                    })
                })
                .filter(|word| {
                    search_word
                        .chars()
                        .skip(usize::from(first_char.is_some()))
                        .unique()
                        .all(|c| {
                            search_word
                                .chars()
                                .filter(|search_word_char| *search_word_char == c)
                                .count()
                                == word.chars().filter(|word_char| *word_char == c).count()
                        })
                })
                .map(String::clone)
                .collect()
        })
}

const SPECIAL_CHARS_PRE: [char; 2] = ['(', '„'];
const SPECIAL_CHARS_POST: [char; 7] = [',', '.', ')', '“', ':', '-', '?'];

fn main() {
    let Some(filepath) = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .filter(|filepath| filepath.is_file()) else {
            eprintln!("USAGE: word-unscrambler [FILE_PATH]");
            return;
        };

    let dictionary = include_str!("german.dic");

    let mut dictionary_map = HashMap::<u64, Vec<String>>::new();

    dictionary.lines().for_each(|word| {
        dictionary_map
            .entry(compute_string_hash(word))
            .or_default()
            .push(word.to_string());
    });

    // Print debug stats about the used associative string hash
    //dbg!(dictionary_map.keys().count());
    //dbg!(dictionary_map.values().map(|words| words.len()).min());
    //dbg!(dictionary_map.values().map(|words| words.len()).max());
    //dbg!(dictionary_map.values().map(|words| words.len()).sum::<usize>() / dictionary_map.values().count());

    let encrypted_news = std::fs::read_to_string(filepath).expect("Failed to read in file!");

    let special_chars: Vec<_> = SPECIAL_CHARS_PRE
        .iter()
        .chain(SPECIAL_CHARS_POST.iter())
        .copied()
        .collect();

    encrypted_news.lines().for_each(|line| {
        line.split_ascii_whitespace().for_each(|encrypted_word| {
            let clean_encrypted_word = encrypted_word
                .chars()
                .filter(|c| !special_chars.contains(c))
                .collect::<String>();

            let word_candidates = {
                let mut word_candidates =
                    find_word_candidates(&dictionary_map, &clean_encrypted_word, None);

                if word_candidates.is_empty() {
                    word_candidates = find_word_candidates(
                        &dictionary_map,
                        &clean_encrypted_word.to_lowercase(),
                        clean_encrypted_word.chars().find(|c| c.is_uppercase()),
                    );

                    for word in &mut word_candidates {
                        if let Some(first_char) = word.get_mut(..1) {
                            first_char.make_ascii_uppercase();
                        }
                    }
                }
                word_candidates
            };

            encrypted_word
                .chars()
                .filter(|c| SPECIAL_CHARS_PRE.contains(c))
                .for_each(|prefix| {
                    print!("{prefix}");
                });

            match &word_candidates[..] {
                [] => {
                    let mut chars = clean_encrypted_word.chars().collect::<Vec<_>>();
                    chars.sort_unstable();
                    print!("`{}`", chars.into_iter().collect::<String>());
                }
                [word_candidate] => {
                    print!("{word_candidate}");
                }
                word_candidates => {
                    print!("{word_candidates:?}");
                }
            };

            encrypted_word
                .chars()
                .filter(|c| SPECIAL_CHARS_POST.contains(c))
                .for_each(|postfix| {
                    print!("{postfix}");
                });

            print!(" ");
        });
        println!();
    });
}