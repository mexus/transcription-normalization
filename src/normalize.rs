use crate::language::Language;
use crate::numbers;
use crate::token::{Canonical, Fragment, Token};
use crate::word::Word;

/// Punctuation that acts as a word separator when it appears between letters
/// (e.g. `Яндекс.Колонку` from a transcriber joining names by `.`). Apostrophe
/// is intentionally absent so English contractions stay glued.
const SPLIT_PUNCT: &[char] = &[
    '.', ',', '!', '?', ';', ':', '"', '«', '»', '(', ')', '[', ']',
    '\u{201C}', '\u{201D}', '…',
];

/// Boundary-only punctuation: stripped from the start/end of a piece after
/// splitting. Covers the apostrophes that `SPLIT_PUNCT` deliberately omits.
const TRIM_PUNCT: &[char] = &['\'', '\u{2018}', '\u{2019}'];

pub fn tokenize(words: &[Word], lang: Language) -> Vec<Token> {
    let fragments = build_fragments(words);
    group_into_tokens(&fragments, lang, words)
}

fn build_fragments(words: &[Word]) -> Vec<Fragment> {
    let mut out = Vec::new();
    let mut buf: Vec<String> = Vec::new();
    for (idx, w) in words.iter().enumerate() {
        for piece in w.text.split_whitespace() {
            buf.clear();
            normalize_into(piece, &mut buf);
            for text in buf.drain(..) {
                out.push(Fragment { word_index: idx, text });
            }
        }
    }
    out
}

fn normalize_into(s: &str, out: &mut Vec<String>) {
    let lower = s.to_lowercase();
    let folded: String = lower.chars().map(|c| if c == 'ё' { 'е' } else { c }).collect();
    for raw_piece in folded.split(|c: char| SPLIT_PUNCT.contains(&c)) {
        let piece = raw_piece.trim_matches(|c: char| TRIM_PUNCT.contains(&c));
        if piece.is_empty() {
            continue;
        }
        if let Some(split) = split_digit_hyphen_run(piece) {
            out.extend(split);
        } else {
            out.push(piece.to_string());
        }
    }
}

/// If `s` looks like `digits(-digits)+` (e.g. ASR output `926-52511-17`),
/// return the digit groups as separate pieces; otherwise None.
/// Mixed forms like `100-летие` or `вице-президент` are left intact so the
/// downstream merge logic can handle them.
fn split_digit_hyphen_run(s: &str) -> Option<Vec<String>> {
    if !s.contains('-') {
        return None;
    }
    let parts: Vec<&str> = s.split('-').filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return None;
    }
    if !parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
        return None;
    }
    Some(parts.into_iter().map(|p| p.to_string()).collect())
}

fn group_into_tokens(frags: &[Fragment], lang: Language, words: &[Word]) -> Vec<Token> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < frags.len() {
        if let Some((n, value)) = numbers::try_consume(lang, &frags[i..])
            && n >= 1
        {
            let raw = frags[i..i + n]
                .iter()
                .map(|f| f.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let mut word_indices: Vec<usize> =
                frags[i..i + n].iter().map(|f| f.word_index).collect();
            word_indices.dedup();
            let (start, end) = span_over_words(&word_indices, words);
            out.push(Token {
                word_indices,
                raw,
                canonical: Canonical::Number(value),
                start,
                end,
            });
            i += n;
            continue;
        }
        let frag = &frags[i];
        let raw = frag.text.clone();
        let canonical_text: String = raw.chars().filter(|&c| c != '-').collect();
        let w = &words[frag.word_index];
        out.push(Token {
            word_indices: vec![frag.word_index],
            raw,
            canonical: Canonical::Word(canonical_text),
            start: w.start,
            end: w.end,
        });
        i += 1;
    }
    out
}

fn span_over_words(indices: &[usize], words: &[Word]) -> (f64, f64) {
    let mut start = f64::INFINITY;
    let mut end = f64::NEG_INFINITY;
    for &i in indices {
        let w = &words[i];
        if w.start < start {
            start = w.start;
        }
        if w.end > end {
            end = w.end;
        }
    }
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(text: &str) -> Word {
        Word::new(text, 0.0, 0.0)
    }

    #[test]
    fn empty() {
        assert!(tokenize(&[], Language::English).is_empty());
    }

    #[test]
    fn lowercase_and_punct_strip() {
        let toks = tokenize(&[w("Hello,"), w("World!")], Language::English);
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].canonical, Canonical::Word("hello".into()));
        assert_eq!(toks[1].canonical, Canonical::Word("world".into()));
    }

    #[test]
    fn keeps_hyphen_in_raw_but_strips_in_canonical() {
        let toks = tokenize(&[w("вице-президент")], Language::Russian);
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].raw, "вице-президент");
        assert_eq!(toks[0].canonical, Canonical::Word("вицепрезидент".into()));
    }

    #[test]
    fn yo_folds_to_e() {
        let toks = tokenize(&[w("ёлка")], Language::Russian);
        assert_eq!(toks[0].canonical, Canonical::Word("елка".into()));
    }

    #[test]
    fn collapses_number_run() {
        let toks = tokenize(&[w("сто"), w("двадцать"), w("три"), w("рубля")], Language::Russian);
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].canonical, Canonical::Number(123));
        assert_eq!(toks[0].word_indices, vec![0, 1, 2]);
        assert_eq!(toks[1].canonical, Canonical::Word("рубля".into()));
        assert_eq!(toks[1].word_indices, vec![3]);
    }

    #[test]
    fn digit_word_yields_number_token() {
        let toks = tokenize(&[w("123")], Language::English);
        assert_eq!(toks[0].canonical, Canonical::Number(123));
    }

    #[test]
    fn interior_punct_splits_glued_words() {
        let toks = tokenize(&[w("Яндекс.Колонку")], Language::Russian);
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].canonical, Canonical::Word("яндекс".into()));
        assert_eq!(toks[1].canonical, Canonical::Word("колонку".into()));
    }

    #[test]
    fn digit_hyphen_run_splits_into_numbers() {
        let toks = tokenize(&[w("926-52511-17")], Language::Russian);
        assert_eq!(toks.len(), 3);
        assert_eq!(toks[0].canonical, Canonical::Number(926));
        assert_eq!(toks[1].canonical, Canonical::Number(52511));
        assert_eq!(toks[2].canonical, Canonical::Number(17));
    }

    #[test]
    fn mixed_hyphen_word_does_not_split() {
        let toks = tokenize(&[w("100-летие")], Language::Russian);
        assert_eq!(toks.len(), 1);
        assert_eq!(toks[0].canonical, Canonical::Word("100летие".into()));
    }

    #[test]
    fn multi_word_input_split_on_whitespace() {
        // A Word whose text already contains whitespace splits into multiple fragments
        // sharing the same source word_index.
        let toks = tokenize(&[w("hello world")], Language::English);
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].word_indices, vec![0]);
        assert_eq!(toks[1].word_indices, vec![0]);
    }
}
