use std::{collections::HashMap, io::Write};

use itertools::Itertools;

fn associative_hash(s: &[u8]) -> u64 {
    s.iter()
        .copied()
        .map(u64::from)
        .fold(0_u64, u64::wrapping_add)
}

fn find_word_candidates(
    dictionary_map: &HashMap<u64, Vec<String>>,
    search_word: &str,
    first_char: Option<char>,
) -> Vec<String> {
    dictionary_map
        .get(&associative_hash(search_word.as_bytes()))
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

static DICTIONARY: once_cell::sync::Lazy<HashMap<u64, Vec<String>>> =
    once_cell::sync::Lazy::new(|| {
        let dictionary = include_str!("german.dic");

        let mut dictionary_map = HashMap::<u64, Vec<String>>::new();

        dictionary.lines().for_each(|word| {
            dictionary_map
                .entry(associative_hash(word.as_bytes()))
                .or_default()
                .push(word.to_string());
        });

        dictionary_map
    });

static SPECIAL_CHARS: once_cell::sync::Lazy<Vec<char>> = once_cell::sync::Lazy::new(|| {
    SPECIAL_CHARS_PRE
        .iter()
        .chain(SPECIAL_CHARS_POST.iter())
        .copied()
        .collect()
});

fn unscramble_line(scrambled_line: &str, output: &mut impl Write) {
    let mut first_word = true;
    for scrambled_word in scrambled_line.split_ascii_whitespace() {
        if !first_word {
            write!(output, " ").unwrap();
        }

        let clean_scrambled_word = scrambled_word
            .chars()
            .filter(|c| !SPECIAL_CHARS.contains(c))
            .collect::<String>();

        let word_candidates = {
            let mut word_candidates =
                find_word_candidates(&DICTIONARY, &clean_scrambled_word, None);

            if word_candidates.is_empty() {
                word_candidates = find_word_candidates(
                    &DICTIONARY,
                    &clean_scrambled_word.to_lowercase(),
                    clean_scrambled_word.chars().find(|c| c.is_uppercase()),
                );

                for word in &mut word_candidates {
                    if let Some(first_char) = word.get_mut(..1) {
                        first_char.make_ascii_uppercase();
                    }
                }
            }
            word_candidates
        };

        scrambled_word
            .chars()
            .filter(|c| SPECIAL_CHARS_PRE.contains(c))
            .for_each(|prefix| {
                write!(output, "{prefix}").unwrap();
            });

        match &word_candidates[..] {
            [] => {
                let mut chars = clean_scrambled_word.chars().collect::<Vec<_>>();
                chars.sort_unstable();
                write!(output, "`{}`", chars.into_iter().collect::<String>()).unwrap();
            }
            [word_candidate] => {
                write!(output, "{word_candidate}").unwrap();
            }
            word_candidates => {
                write!(output, "{word_candidates:?}").unwrap();
            }
        };

        scrambled_word
            .chars()
            .filter(|c| SPECIAL_CHARS_POST.contains(c))
            .for_each(|postfix| {
                write!(output, "{postfix}").unwrap();
            });

        first_word = false;
    }
}

fn unscramble(scrambled_string: &str, output: &mut impl Write) {
    let mut first_line = true;
    for scrambled_line in scrambled_string.lines() {
        if !first_line {
            writeln!(output).unwrap();
        }

        unscramble_line(scrambled_line, output);

        first_line = false;
    }
}

fn main() {
    let Some(filepath) = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .filter(|filepath| filepath.is_file()) else {
            eprintln!("USAGE: word-unscrambler [FILE_PATH]");
            return;
        };

    // Print debug stats about the used associative string hash
    //dbg!(DICTIONARY.keys().count());
    //dbg!(DICTIONARY.values().map(|words| words.len()).min());
    //dbg!(DICTIONARY.values().map(|words| words.len()).max());
    //dbg!(DICTIONARY.values().map(|words| words.len()).sum::<usize>() / DICTIONARY.values().count());

    let scrambled_string = std::fs::read_to_string(filepath).expect("Failed to read in file!");
    unscramble(&scrambled_string, &mut std::io::stdout());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_associative_hash() {
        const TEST_STRING: &[u8] = b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";
        let mut sorted_string: Vec<_> = TEST_STRING.to_vec();
        sorted_string.sort_unstable();
        assert_eq!(
            associative_hash(TEST_STRING),
            associative_hash(&sorted_string)
        );
    }

    #[test]
    fn test_unscrable() {
        const TEST_STRING: &str = "eiD rüedW dse cnnesheM its atusar.tanbn eSi uz eahntc dnu uz shcenztü tis tincufhpegrlV ealrl iesnatclhat .eawltG";
        const EXPECTED_STRING: &str = r#"Die Würde des Menschen ist unantastbar. ["Sei", "Sie"] zu achten und zu schützen ist Verpflichtung aller ["atlantische", "staatlichen"] Gewalt."#;
        let mut output_string = Vec::new();
        unscramble(TEST_STRING, &mut output_string);
        assert_eq!(String::from_utf8(output_string).unwrap(), EXPECTED_STRING);
    }
}
