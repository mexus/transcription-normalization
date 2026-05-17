use transcription_normalization::{compare, Canonical, Language, Op, Word};

fn w(text: &str) -> Word {
    Word::new(text, 0.0, 0.0)
}

fn words(texts: &[&str]) -> Vec<Word> {
    texts.iter().map(|t| w(t)).collect()
}

#[test]
fn perfect_english_match() {
    let r = words(&["The", "quick", "brown", "fox"]);
    let h = words(&["the", "quick", "brown", "fox"]);
    let result = compare(&r, &h, Language::English);
    assert_eq!(result.errors(), 0);
    assert_eq!(result.wer(), 0.0);
}

#[test]
fn english_numeral_variants_match() {
    // digit form vs word form vs ordinal — all collapse to Number(100)
    let r = words(&["100", "apples"]);
    let h = words(&["one", "hundred", "apples"]);
    let result = compare(&r, &h, Language::English);
    assert_eq!(result.errors(), 0, "ops: {:?}", result.ops);

    let r2 = words(&["hundredth", "apple"]);
    let h2 = words(&["100", "apple"]);
    let result2 = compare(&r2, &h2, Language::English);
    assert_eq!(result2.errors(), 0);
}

#[test]
fn russian_numeral_variants_match() {
    let r = words(&["сто", "двадцать", "три", "рубля"]);
    let h = words(&["123", "рубля"]);
    let result = compare(&r, &h, Language::Russian);
    assert_eq!(result.errors(), 0, "ops: {:?}", result.ops);
}

#[test]
fn russian_ordinal_matches_cardinal() {
    // "первого января" (first of January) ↔ "1 января" — both should be a perfect match
    let r = words(&["первого", "января"]);
    let h = words(&["1", "января"]);
    let result = compare(&r, &h, Language::Russian);
    assert_eq!(result.errors(), 0, "ops: {:?}", result.ops);
}

#[test]
fn hyphen_variants_all_equivalent() {
    let joined = words(&["вицепрезидент", "сказал"]);
    let hyphenated = words(&["вице-президент", "сказал"]);
    let split = words(&["вице", "президент", "сказал"]);

    for r in &[&joined, &hyphenated, &split] {
        for h in &[&joined, &hyphenated, &split] {
            let result = compare(r, h, Language::Russian);
            assert_eq!(
                result.errors(),
                0,
                "expected 0 errors between {:?} and {:?}, got ops {:?}",
                r,
                h,
                result.ops
            );
        }
    }
}

#[test]
fn yo_fold_does_not_penalize() {
    let r = words(&["ёлка", "стоит"]);
    let h = words(&["елка", "стоит"]);
    let result = compare(&r, &h, Language::Russian);
    assert_eq!(result.errors(), 0);
}

#[test]
fn substitution_is_counted() {
    let r = words(&["the", "cat", "sat"]);
    let h = words(&["the", "dog", "sat"]);
    let result = compare(&r, &h, Language::English);
    assert_eq!(result.substitutions(), 1);
    assert_eq!(result.insertions(), 0);
    assert_eq!(result.deletions(), 0);
    assert!((result.wer() - 1.0 / 3.0).abs() < 1e-9);
}

#[test]
fn insertion_and_deletion_counted() {
    let r = words(&["the", "cat", "sat", "down"]);
    let h = words(&["the", "sat"]);
    let result = compare(&r, &h, Language::English);
    // Two deletions: "cat" and "down"
    assert_eq!(result.deletions(), 2);
    assert_eq!(result.errors(), 2);
}

#[test]
fn token_word_indices_point_back_to_input() {
    let r = words(&["сто", "двадцать", "три"]);
    let h = words(&["123"]);
    let result = compare(&r, &h, Language::Russian);
    assert_eq!(result.errors(), 0);
    // ref side should be a single Number(123) token spanning all 3 input words.
    assert_eq!(result.ref_tokens.len(), 1);
    assert_eq!(result.ref_tokens[0].canonical, Canonical::Number(123));
    assert_eq!(result.ref_tokens[0].word_indices, vec![0, 1, 2]);
}

#[test]
fn realistic_mixed_pair() {
    // A messy ref vs a clean hyp — verify the metric is sane (not 0, not crazy).
    let r = words(&[
        "Президент", "сказал,", "что", "к", "2030", "году",
        "вице-президент", "посетит", "сто", "стран.",
    ]);
    let h = words(&[
        "президент", "сказал", "что", "к", "две", "тысячи", "тридцатому",
        "году", "вицепрезидент", "посетит", "100", "стран",
    ]);
    let result = compare(&r, &h, Language::Russian);
    // Numerals + hyphen variants should all match. Only difference might be due to "2030" vs "две тысячи тридцатому":
    // - "2030" → Number(2030) on ref side
    // - "две тысячи тридцатому" → Number(2030) on hyp side (cardinal 2 + scale 1000 + ordinal 30 → 2030)
    // and "сто стран" → 100; "100 стран" → 100. All match.
    assert_eq!(
        result.errors(),
        0,
        "expected 0 errors, got {} (ops: {:?})",
        result.errors(),
        result.ops
    );
}

#[test]
fn hyphen_merge_produces_match_op_with_correct_ranges() {
    let r = words(&["вице-президент"]);
    let h = words(&["вице", "президент"]);
    let result = compare(&r, &h, Language::Russian);
    assert_eq!(result.ops.len(), 1);
    match &result.ops[0] {
        Op::Match { ref_range, hyp_range } => {
            assert_eq!(*ref_range, 0..1);
            assert_eq!(*hyp_range, 0..2);
        }
        other => panic!("expected Match, got {:?}", other),
    }
}
