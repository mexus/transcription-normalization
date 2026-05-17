use std::collections::HashMap;
use std::sync::LazyLock;

use crate::token::Fragment;

use super::{Atom, grammar_parse};

static TABLE: LazyLock<HashMap<&'static str, Atom>> = LazyLock::new(build_table);

fn build_table() -> HashMap<&'static str, Atom> {
    let mut m = HashMap::new();

    m.insert("zero", Atom::Zero);

    let units: &[(&str, &str, i64)] = &[
        ("one", "first", 1),
        ("two", "second", 2),
        ("three", "third", 3),
        ("four", "fourth", 4),
        ("five", "fifth", 5),
        ("six", "sixth", 6),
        ("seven", "seventh", 7),
        ("eight", "eighth", 8),
        ("nine", "ninth", 9),
    ];
    for (card, ord, n) in units {
        m.insert(*card, Atom::Unit(*n));
        m.insert(*ord, Atom::Unit(*n));
    }

    let teens: &[(&str, &str, i64)] = &[
        ("ten", "tenth", 10),
        ("eleven", "eleventh", 11),
        ("twelve", "twelfth", 12),
        ("thirteen", "thirteenth", 13),
        ("fourteen", "fourteenth", 14),
        ("fifteen", "fifteenth", 15),
        ("sixteen", "sixteenth", 16),
        ("seventeen", "seventeenth", 17),
        ("eighteen", "eighteenth", 18),
        ("nineteen", "nineteenth", 19),
    ];
    for (card, ord, n) in teens {
        m.insert(*card, Atom::Teen(*n));
        m.insert(*ord, Atom::Teen(*n));
    }

    let tens: &[(&str, &str, i64)] = &[
        ("twenty", "twentieth", 20),
        ("thirty", "thirtieth", 30),
        ("forty", "fortieth", 40),
        ("fifty", "fiftieth", 50),
        ("sixty", "sixtieth", 60),
        ("seventy", "seventieth", 70),
        ("eighty", "eightieth", 80),
        ("ninety", "ninetieth", 90),
    ];
    for (card, ord, n) in tens {
        m.insert(*card, Atom::Tens(*n));
        m.insert(*ord, Atom::Tens(*n));
    }

    m.insert("hundred", Atom::HundredScale);
    m.insert("hundredth", Atom::HundredScale);

    m.insert("thousand", Atom::Scale(1_000));
    m.insert("thousandth", Atom::Scale(1_000));
    m.insert("million", Atom::Scale(1_000_000));
    m.insert("millionth", Atom::Scale(1_000_000));
    m.insert("billion", Atom::Scale(1_000_000_000));
    m.insert("billionth", Atom::Scale(1_000_000_000));

    m.insert("and", Atom::Glue);

    m
}

pub(crate) fn try_consume(frags: &[Fragment]) -> Option<(usize, i64)> {
    if let Some(first) = frags.first()
        && let Ok(n) = first.text.parse::<i64>()
    {
        return Some((1, n));
    }
    parse_word_run(frags)
}

fn parse_word_run(frags: &[Fragment]) -> Option<(usize, i64)> {
    let mut atoms: Vec<Atom> = Vec::new();
    let mut consumed = 0usize;
    let mut last_real_consumed = 0usize;
    let mut last_real_atoms_len = 0usize;

    for frag in frags {
        let pieces: Vec<&str> = frag.text.split('-').filter(|s| !s.is_empty()).collect();
        if pieces.is_empty() {
            break;
        }

        let mut frag_atoms = Vec::with_capacity(pieces.len());
        let mut ok = true;
        for piece in &pieces {
            if let Some(&atom) = TABLE.get(piece) {
                frag_atoms.push(atom);
            } else {
                ok = false;
                break;
            }
        }

        if !ok {
            break;
        }

        let has_real = frag_atoms.iter().any(|a| !matches!(a, Atom::Glue));
        if !has_real && atoms.is_empty() {
            break;
        }

        atoms.extend(frag_atoms);
        consumed += 1;
        if has_real {
            last_real_consumed = consumed;
            last_real_atoms_len = atoms.len();
        }
    }

    atoms.truncate(last_real_atoms_len);

    if atoms.is_empty() {
        return None;
    }

    grammar_parse(&atoms).map(|v| (last_real_consumed, v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frags(words: &[&str]) -> Vec<Fragment> {
        words
            .iter()
            .enumerate()
            .map(|(i, w)| Fragment { word_index: i, text: w.to_string() })
            .collect()
    }

    fn parse(words: &[&str]) -> Option<(usize, i64)> {
        try_consume(&frags(words))
    }

    #[test]
    fn zero() {
        assert_eq!(parse(&["zero"]), Some((1, 0)));
    }

    #[test]
    fn single_digit() {
        assert_eq!(parse(&["five"]), Some((1, 5)));
        assert_eq!(parse(&["fifth"]), Some((1, 5)));
    }

    #[test]
    fn teen() {
        assert_eq!(parse(&["thirteen"]), Some((1, 13)));
        assert_eq!(parse(&["twelfth"]), Some((1, 12)));
    }

    #[test]
    fn compound_tens() {
        assert_eq!(parse(&["twenty", "one"]), Some((2, 21)));
        assert_eq!(parse(&["twenty-one"]), Some((1, 21)));
        assert_eq!(parse(&["twenty-first"]), Some((1, 21)));
    }

    #[test]
    fn hundreds() {
        assert_eq!(parse(&["one", "hundred"]), Some((2, 100)));
        assert_eq!(parse(&["one", "hundred", "and", "twenty", "three"]), Some((5, 123)));
        assert_eq!(parse(&["five", "hundred"]), Some((2, 500)));
        assert_eq!(parse(&["hundredth"]), Some((1, 100)));
    }

    #[test]
    fn scales() {
        assert_eq!(parse(&["thousand"]), Some((1, 1000)));
        assert_eq!(parse(&["thousandth"]), Some((1, 1000)));
        assert_eq!(parse(&["two", "thousand"]), Some((2, 2_000)));
        assert_eq!(parse(&["two", "million", "three"]), Some((3, 2_000_003)));
        assert_eq!(
            parse(&["one", "hundred", "twenty", "three", "thousand", "four", "hundred", "fifty", "six"]),
            Some((9, 123_456))
        );
    }

    #[test]
    fn digit_literal() {
        assert_eq!(parse(&["123"]), Some((1, 123)));
        assert_eq!(parse(&["0"]), Some((1, 0)));
    }

    #[test]
    fn stops_at_non_number() {
        assert_eq!(parse(&["five", "apples"]), Some((1, 5)));
        assert_eq!(parse(&["five", "hundred", "apples"]), Some((2, 500)));
    }

    #[test]
    fn junk_returns_none() {
        assert_eq!(parse(&["apples"]), None);
        assert_eq!(parse(&[]), None);
    }

    #[test]
    fn trailing_and_does_not_consume() {
        // "five and" should parse as 5 but only consume 1 fragment (the trailing "and" gets dropped).
        let result = parse(&["five", "and"]);
        assert!(matches!(result, Some((_, 5))));
    }
}
