use crate::language::Language;
use crate::numbers;
use crate::token::{Canonical, Fragment, Token};
use crate::word::Word;

const PUNCT: &[char] = &[
    '.', ',', '!', '?', ';', ':', '"', '«', '»', '(', ')', '[', ']',
    '\'', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '…',
];

pub fn tokenize(words: &[Word], lang: Language) -> Vec<Token> {
    let fragments = build_fragments(words);
    group_into_tokens(&fragments, lang)
}

fn build_fragments(words: &[Word]) -> Vec<Fragment> {
    let mut out = Vec::new();
    for (idx, w) in words.iter().enumerate() {
        for piece in w.text.split_whitespace() {
            let normalized = normalize_fragment(piece);
            if !normalized.is_empty() {
                out.push(Fragment { word_index: idx, text: normalized });
            }
        }
    }
    out
}

fn normalize_fragment(s: &str) -> String {
    let lower = s.to_lowercase();
    let folded: String = lower.chars().map(|c| if c == 'ё' { 'е' } else { c }).collect();
    folded
        .trim_matches(|c: char| PUNCT.contains(&c))
        .to_string()
}

fn group_into_tokens(frags: &[Fragment], lang: Language) -> Vec<Token> {
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
            out.push(Token {
                word_indices,
                raw,
                canonical: Canonical::Number(value),
            });
            i += n;
            continue;
        }
        let frag = &frags[i];
        let raw = frag.text.clone();
        let canonical_text: String = raw.chars().filter(|&c| c != '-').collect();
        out.push(Token {
            word_indices: vec![frag.word_index],
            raw,
            canonical: Canonical::Word(canonical_text),
        });
        i += 1;
    }
    out
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
    fn multi_word_input_split_on_whitespace() {
        // A Word whose text already contains whitespace splits into multiple fragments
        // sharing the same source word_index.
        let toks = tokenize(&[w("hello world")], Language::English);
        assert_eq!(toks.len(), 2);
        assert_eq!(toks[0].word_indices, vec![0]);
        assert_eq!(toks[1].word_indices, vec![0]);
    }
}
