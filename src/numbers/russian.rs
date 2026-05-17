use std::collections::HashMap;
use std::sync::LazyLock;

use crate::token::Fragment;

use super::{Atom, longest_valid_prefix};

static TABLE: LazyLock<HashMap<String, Atom>> = LazyLock::new(build_table);

fn build_table() -> HashMap<String, Atom> {
    let mut m: HashMap<String, Atom> = HashMap::new();

    add_zero(&mut m, &["ноль", "нуль", "ноля", "нуля", "нолю", "нулю", "нолем", "нулем", "ноле", "нуле"]);

    add_unit(&mut m, 1, &[
        "один", "одна", "одно", "одни",
        "одного", "одной", "одних",
        "одному", "одним",
        "одними",
        "одном",
        "одну",
    ]);
    add_unit(&mut m, 2, &["два", "две", "двух", "двум", "двумя"]);
    add_unit(&mut m, 3, &["три", "трех", "трем", "тремя"]);
    add_unit(&mut m, 4, &["четыре", "четырех", "четырем", "четырьмя"]);
    add_unit(&mut m, 5, &["пять", "пяти", "пятью"]);
    add_unit(&mut m, 6, &["шесть", "шести", "шестью"]);
    add_unit(&mut m, 7, &["семь", "семи", "семью"]);
    add_unit(&mut m, 8, &["восемь", "восьми", "восемью", "восьмью"]);
    add_unit(&mut m, 9, &["девять", "девяти", "девятью"]);

    add_teen(&mut m, 10, &["десять", "десяти", "десятью"]);
    add_teen(&mut m, 11, &["одиннадцать", "одиннадцати", "одиннадцатью"]);
    add_teen(&mut m, 12, &["двенадцать", "двенадцати", "двенадцатью"]);
    add_teen(&mut m, 13, &["тринадцать", "тринадцати", "тринадцатью"]);
    add_teen(&mut m, 14, &["четырнадцать", "четырнадцати", "четырнадцатью"]);
    add_teen(&mut m, 15, &["пятнадцать", "пятнадцати", "пятнадцатью"]);
    add_teen(&mut m, 16, &["шестнадцать", "шестнадцати", "шестнадцатью"]);
    add_teen(&mut m, 17, &["семнадцать", "семнадцати", "семнадцатью"]);
    add_teen(&mut m, 18, &["восемнадцать", "восемнадцати", "восемнадцатью"]);
    add_teen(&mut m, 19, &["девятнадцать", "девятнадцати", "девятнадцатью"]);

    add_tens(&mut m, 20, &["двадцать", "двадцати", "двадцатью"]);
    add_tens(&mut m, 30, &["тридцать", "тридцати", "тридцатью"]);
    add_tens(&mut m, 40, &["сорок", "сорока"]);
    add_tens(&mut m, 50, &["пятьдесят", "пятидесяти", "пятьюдесятью"]);
    add_tens(&mut m, 60, &["шестьдесят", "шестидесяти", "шестьюдесятью"]);
    add_tens(&mut m, 70, &["семьдесят", "семидесяти", "семьюдесятью"]);
    add_tens(&mut m, 80, &["восемьдесят", "восьмидесяти", "восемьюдесятью"]);
    add_tens(&mut m, 90, &["девяносто", "девяноста"]);

    add_hundreds(&mut m, 100, &["сто", "ста"]);
    add_hundreds(&mut m, 200, &["двести", "двухсот", "двумстам", "двумястами", "двухстах"]);
    add_hundreds(&mut m, 300, &["триста", "трехсот", "тремстам", "тремястами", "трехстах"]);
    add_hundreds(&mut m, 400, &["четыреста", "четырехсот", "четыремстам", "четырьмястами", "четырехстах"]);
    add_hundreds(&mut m, 500, &["пятьсот", "пятисот", "пятистам", "пятьюстами", "пятистах"]);
    add_hundreds(&mut m, 600, &["шестьсот", "шестисот", "шестистам", "шестьюстами", "шестистах"]);
    add_hundreds(&mut m, 700, &["семьсот", "семисот", "семистам", "семьюстами", "семистах"]);
    add_hundreds(&mut m, 800, &["восемьсот", "восьмисот", "восьмистам", "восемьюстами", "восьмистах"]);
    add_hundreds(&mut m, 900, &["девятьсот", "девятисот", "девятистам", "девятьюстами", "девятистах"]);

    add_scale(&mut m, 1_000, &[
        "тысяча", "тысячи", "тысяч", "тысяче", "тысячу", "тысячей", "тысячью",
        "тысячам", "тысячами", "тысячах",
    ]);
    add_scale(&mut m, 1_000_000, &[
        "миллион", "миллиона", "миллионов", "миллиону", "миллионе",
        "миллионы", "миллионам", "миллионами", "миллионах",
    ]);
    add_scale(&mut m, 1_000_000_000, &[
        "миллиард", "миллиарда", "миллиардов", "миллиарду", "миллиарде",
        "миллиарды", "миллиардам", "миллиардами", "миллиардах",
    ]);

    add_ord_y(&mut m, 1, "перв");
    add_ord_oy(&mut m, 2, "втор");
    add_ord_iy(&mut m, 3, "трет");
    add_ord_y(&mut m, 4, "четверт");
    add_ord_y(&mut m, 5, "пят");
    add_ord_oy(&mut m, 6, "шест");
    add_ord_oy(&mut m, 7, "седьм");
    add_ord_oy(&mut m, 8, "восьм");
    add_ord_y(&mut m, 9, "девят");
    add_ord_y(&mut m, 10, "десят");
    add_ord_y(&mut m, 11, "одиннадцат");
    add_ord_y(&mut m, 12, "двенадцат");
    add_ord_y(&mut m, 13, "тринадцат");
    add_ord_y(&mut m, 14, "четырнадцат");
    add_ord_y(&mut m, 15, "пятнадцат");
    add_ord_y(&mut m, 16, "шестнадцат");
    add_ord_y(&mut m, 17, "семнадцат");
    add_ord_y(&mut m, 18, "восемнадцат");
    add_ord_y(&mut m, 19, "девятнадцат");
    add_ord_y(&mut m, 20, "двадцат");
    add_ord_y(&mut m, 30, "тридцат");
    add_ord_oy(&mut m, 40, "сороков");
    add_ord_y(&mut m, 50, "пятидесят");
    add_ord_y(&mut m, 60, "шестидесят");
    add_ord_y(&mut m, 70, "семидесят");
    add_ord_y(&mut m, 80, "восьмидесят");
    add_ord_y(&mut m, 90, "девяност");
    add_ord_y(&mut m, 100, "сот");
    add_ord_y(&mut m, 200, "двухсот");
    add_ord_y(&mut m, 300, "трехсот");
    add_ord_y(&mut m, 400, "четырехсот");
    add_ord_y(&mut m, 500, "пятисот");
    add_ord_y(&mut m, 600, "шестисот");
    add_ord_y(&mut m, 700, "семисот");
    add_ord_y(&mut m, 800, "восьмисот");
    add_ord_y(&mut m, 900, "девятисот");

    add_ord_y_as_scale(&mut m, 1_000, "тысячн");
    add_ord_y_as_scale(&mut m, 1_000_000, "миллионн");
    add_ord_y_as_scale(&mut m, 1_000_000_000, "миллиардн");

    m
}

fn add_zero(m: &mut HashMap<String, Atom>, forms: &[&str]) {
    for f in forms {
        m.insert(f.to_string(), Atom::Zero);
    }
}

fn add_unit(m: &mut HashMap<String, Atom>, n: i64, forms: &[&str]) {
    for f in forms {
        m.insert(f.to_string(), Atom::Unit(n));
    }
}

fn add_teen(m: &mut HashMap<String, Atom>, n: i64, forms: &[&str]) {
    for f in forms {
        m.insert(f.to_string(), Atom::Teen(n));
    }
}

fn add_tens(m: &mut HashMap<String, Atom>, n: i64, forms: &[&str]) {
    for f in forms {
        m.insert(f.to_string(), Atom::Tens(n));
    }
}

fn add_hundreds(m: &mut HashMap<String, Atom>, n: i64, forms: &[&str]) {
    for f in forms {
        m.insert(f.to_string(), Atom::Hundreds(n));
    }
}

fn add_scale(m: &mut HashMap<String, Atom>, v: i64, forms: &[&str]) {
    for f in forms {
        m.insert(f.to_string(), Atom::Scale(v));
    }
}

const ORD_Y_SUFFIXES: &[&str] = &[
    "ый", "ая", "ое", "ые",
    "ого", "ой", "ых",
    "ому", "ым",
    "ом", "ыми",
    "ую",
];

const ORD_OY_SUFFIXES: &[&str] = &[
    "ой", "ая", "ое", "ые",
    "ого", "ой", "ых",
    "ому", "ым",
    "ом", "ыми",
    "ую",
];

const ORD_IY_SUFFIXES: &[&str] = &[
    "ий", "ья", "ье", "ьи",
    "ьего", "ьей", "ьих",
    "ьему", "ьим",
    "ьем", "ьими",
    "ью",
];

fn atom_for_value(n: i64) -> Atom {
    match n {
        0 => Atom::Zero,
        1..=9 => Atom::Unit(n),
        10..=19 => Atom::Teen(n),
        20 | 30 | 40 | 50 | 60 | 70 | 80 | 90 => Atom::Tens(n),
        100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900 => Atom::Hundreds(n),
        _ => unreachable!("non-ordinal value {n}"),
    }
}

fn add_ord_y(m: &mut HashMap<String, Atom>, value: i64, stem: &str) {
    let atom = atom_for_value(value);
    for suf in ORD_Y_SUFFIXES {
        m.insert(format!("{stem}{suf}"), atom);
    }
}

fn add_ord_oy(m: &mut HashMap<String, Atom>, value: i64, stem: &str) {
    let atom = atom_for_value(value);
    for suf in ORD_OY_SUFFIXES {
        m.insert(format!("{stem}{suf}"), atom);
    }
}

fn add_ord_iy(m: &mut HashMap<String, Atom>, value: i64, stem: &str) {
    let atom = atom_for_value(value);
    for suf in ORD_IY_SUFFIXES {
        m.insert(format!("{stem}{suf}"), atom);
    }
}

fn add_ord_y_as_scale(m: &mut HashMap<String, Atom>, value: i64, stem: &str) {
    let atom = Atom::Scale(value);
    for suf in ORD_Y_SUFFIXES {
        m.insert(format!("{stem}{suf}"), atom);
    }
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
    let mut frag_atom_ends: Vec<usize> = Vec::new();

    for frag in frags {
        let pieces: Vec<&str> = frag.text.split('-').filter(|s| !s.is_empty()).collect();
        if pieces.is_empty() {
            break;
        }

        let mut frag_atoms = Vec::with_capacity(pieces.len());
        let mut ok = true;
        for piece in &pieces {
            if let Some(&atom) = TABLE.get(*piece) {
                frag_atoms.push(atom);
            } else {
                ok = false;
                break;
            }
        }

        if !ok {
            break;
        }

        atoms.extend(frag_atoms);
        frag_atom_ends.push(atoms.len());
    }

    longest_valid_prefix(&atoms, &frag_atom_ends)
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
        assert_eq!(parse(&["ноль"]), Some((1, 0)));
    }

    #[test]
    fn cardinals() {
        assert_eq!(parse(&["один"]), Some((1, 1)));
        assert_eq!(parse(&["одна"]), Some((1, 1)));
        assert_eq!(parse(&["одну"]), Some((1, 1)));
        assert_eq!(parse(&["два"]), Some((1, 2)));
        assert_eq!(parse(&["две"]), Some((1, 2)));
        assert_eq!(parse(&["двух"]), Some((1, 2)));
        assert_eq!(parse(&["пять"]), Some((1, 5)));
    }

    #[test]
    fn ordinals_collapse_to_value() {
        assert_eq!(parse(&["первый"]), Some((1, 1)));
        assert_eq!(parse(&["первого"]), Some((1, 1)));
        assert_eq!(parse(&["первая"]), Some((1, 1)));
        assert_eq!(parse(&["сотый"]), Some((1, 100)));
        assert_eq!(parse(&["сотая"]), Some((1, 100)));
        assert_eq!(parse(&["сотого"]), Some((1, 100)));
        assert_eq!(parse(&["двадцатого"]), Some((1, 20)));
        // ё-folded form (input is normalized): "трёх" → "трех" → Unit(3)
        assert_eq!(parse(&["трех"]), Some((1, 3)));
    }

    #[test]
    fn compound() {
        assert_eq!(parse(&["сто", "двадцать", "три"]), Some((3, 123)));
        assert_eq!(parse(&["одна", "тысяча", "девятьсот", "девяносто"]), Some((4, 1990)));
        assert_eq!(parse(&["две", "тысячи"]), Some((2, 2_000)));
        assert_eq!(parse(&["сто", "двадцать", "три", "тысячи", "четыреста", "пятьдесят", "шесть"]),
                   Some((7, 123_456)));
    }

    #[test]
    fn digit_literal() {
        assert_eq!(parse(&["123"]), Some((1, 123)));
    }

    #[test]
    fn stops_at_non_number() {
        assert_eq!(parse(&["сто", "рублей"]), Some((1, 100)));
        assert_eq!(parse(&["первого", "января"]), Some((1, 1)));
    }

    #[test]
    fn backtracks_on_invalid_grammar_in_run() {
        // Two sentences worth of number-words glued together by punctuation stripping.
        // Greedy collection swallows the whole run; the grammar rejects it; we must
        // back off to the longest valid fragment-prefix and leave the rest for the
        // next call.
        assert_eq!(
            parse(&["девятьсот", "двадцать", "шесть", "пятьдесят", "два"]),
            Some((3, 926))
        );
        assert_eq!(parse(&["пятьдесят", "два", "пятьсот"]), Some((2, 52)));
        assert_eq!(parse(&["сорок", "четырнадцать"]), Some((1, 40)));
    }

    #[test]
    fn junk_returns_none() {
        assert_eq!(parse(&["яблоки"]), None);
        assert_eq!(parse(&[]), None);
    }
}
