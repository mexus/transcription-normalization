use transcription_normalization::{Canonical, Language, Op, Word, compare};

fn w(text: &str) -> Word {
    Word::new(text, 0.0, 0.0)
}

fn tw(text: &str, start: f64, end: f64) -> Word {
    Word::new(text, start, end)
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
        "Президент",
        "сказал,",
        "что",
        "к",
        "2030",
        "году",
        "вице-президент",
        "посетит",
        "сто",
        "стран.",
    ]);
    let h = words(&[
        "президент",
        "сказал",
        "что",
        "к",
        "две",
        "тысячи",
        "тридцатому",
        "году",
        "вицепрезидент",
        "посетит",
        "100",
        "стран",
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
        Op::Match {
            ref_range,
            hyp_range,
        } => {
            assert_eq!(*ref_range, 0..1);
            assert_eq!(*hyp_range, 0..2);
        }
        other => panic!("expected Match, got {:?}", other),
    }
}

#[test]
fn op_timing_single_token_match() {
    let r = vec![tw("hello", 0.0, 0.5), tw("world", 0.5, 1.0)];
    let h = vec![tw("hello", 0.1, 0.6), tw("world", 0.6, 1.1)];
    let result = compare(&r, &h, Language::English);
    assert_eq!(result.ops.len(), 2);

    let t0 = result.op_timing(&result.ops[0]);
    let rs = t0.ref_span.unwrap();
    let hs = t0.hyp_span.unwrap();
    assert!((rs.start - 0.0).abs() < 1e-9);
    assert!((rs.end - 0.5).abs() < 1e-9);
    assert!((hs.start - 0.1).abs() < 1e-9);
    assert!((hs.end - 0.6).abs() < 1e-9);
}

#[test]
fn op_timing_n_to_m_match_unions_ranges() {
    // ref is one Word "вице-президент" 0.0..1.0
    // hyp is two Words "вице" 0.2..0.6 and "президент" 0.6..1.1
    // The hyphen merge produces a single Match; its ref span should be 0..1.0,
    // its hyp span the union 0.2..1.1.
    let r = vec![tw("вице-президент", 0.0, 1.0)];
    let h = vec![tw("вице", 0.2, 0.6), tw("президент", 0.6, 1.1)];
    let result = compare(&r, &h, Language::Russian);
    assert_eq!(result.ops.len(), 1);
    let t = result.op_timing(&result.ops[0]);
    let rs = t.ref_span.unwrap();
    let hs = t.hyp_span.unwrap();
    assert!((rs.start - 0.0).abs() < 1e-9);
    assert!((rs.end - 1.0).abs() < 1e-9);
    assert!((hs.start - 0.2).abs() < 1e-9);
    assert!((hs.end - 1.1).abs() < 1e-9, "hyp_span.end was {}", hs.end);
}

#[test]
fn op_timing_collapsed_number_spans_all_source_words() {
    let r = vec![
        tw("сто", 0.0, 0.3),
        tw("двадцать", 0.3, 0.7),
        tw("три", 0.7, 1.0),
    ];
    let h = vec![tw("123", 0.05, 0.95)];
    let result = compare(&r, &h, Language::Russian);
    assert_eq!(result.errors(), 0);
    let t = result.op_timing(&result.ops[0]);
    let rs = t.ref_span.unwrap();
    let hs = t.hyp_span.unwrap();
    // Collapsed number on ref side spans the union of all 3 input Words.
    assert!((rs.start - 0.0).abs() < 1e-9);
    assert!((rs.end - 1.0).abs() < 1e-9);
    assert!((hs.start - 0.05).abs() < 1e-9);
    assert!((hs.end - 0.95).abs() < 1e-9);
}

#[test]
fn op_timing_insert_has_no_ref_span() {
    let r = vec![tw("hello", 0.0, 0.5)];
    let h = vec![tw("hello", 0.0, 0.5), tw("world", 0.5, 1.0)];
    let result = compare(&r, &h, Language::English);
    let ins = result
        .ops
        .iter()
        .find(|o| matches!(o, Op::Ins { .. }))
        .unwrap();
    let t = result.op_timing(ins);
    assert!(t.ref_span.is_none());
    let hs = t.hyp_span.unwrap();
    assert!((hs.start - 0.5).abs() < 1e-9);
    assert!((hs.end - 1.0).abs() < 1e-9);
}

#[test]
fn op_timing_delete_has_no_hyp_span() {
    let r = vec![tw("hello", 0.0, 0.5), tw("world", 0.5, 1.0)];
    let h = vec![tw("hello", 0.0, 0.5)];
    let result = compare(&r, &h, Language::English);
    let del = result
        .ops
        .iter()
        .find(|o| matches!(o, Op::Del { .. }))
        .unwrap();
    let t = result.op_timing(del);
    assert!(t.hyp_span.is_none());
    let rs = t.ref_span.unwrap();
    assert!((rs.start - 0.5).abs() < 1e-9);
    assert!((rs.end - 1.0).abs() < 1e-9);
}

#[test]
fn user_supplied_failing_sample() {
    // Real-world sample where alignment was reported to fail. Em-dashes in the
    // original hypothesis display were alignment-gap fillers, not literal ASR
    // output, so they are stripped here.
    let ref_text = "Пополнить телефон друга на сто рублеи. Девятьсот двадцать шесть. \
        Пятьдесят два. Пятьсот одинадцать семнадцать. Купить Яндекс колонку. \
        У вас консьерж сервис в отеле работает целыи день. Часть один. \
        Сезон семь. Однажды в сказке. Сольвычегодск. У тебя есть боицы? \
        Два девятьсот девяносто семь. Восемьсот восемьдесят девять. \
        Сорок четырнадцать. Наиди песню. Две минуты жизни. \
        Хочу пополнить телефон, но не свои номер. Афина. \
        Когда карта Мир будет готова? Включить. \
        Приставка в двадцать ноль ноль. Наиди мультфильм, Губка Боб. \
        Пять, восемьсот сорок пять. Три, Семь ноль девять. Три, шесть, три. \
        Великолепная семерка с Джулианои Мур. Покажи!";

    let hyp_text = "пополнить телефон друга на 100 рублеи 926-52511-17 купить \
        Яндекс.Колонку у вас КАМИСЕЖСЕРВИС в отеле работает целыи день? \
        Часть 1, сезон 7, однажды в сказке Соль вычегоцк \
        У тебя есть боицы? 2-997-889-4014 Даи мне песню \"Две минуты жизни\" \
        Хочу пополнить телефон, но не свои номер. \
        Афина, когда карта мир будет готова? \
        Включить приставку у 2000. Наиди в мультфильм губка Боб. \
        5 845 370 9363 Великолепная семерка с Джулианои Мур, покажи!";

    let r: Vec<Word> = ref_text.split_whitespace().map(w).collect();
    let h: Vec<Word> = hyp_text.split_whitespace().map(w).collect();

    let result = compare(&r, &h, Language::Russian);

    eprintln!("\n=== REF/HYP alignment for failing sample ===");
    eprintln!(
        "ref tokens: {}  hyp tokens: {}",
        result.ref_tokens.len(),
        result.hyp_tokens.len()
    );
    eprintln!(
        "S={} I={} D={}  errors={}  WER={:.3}",
        result.substitutions(),
        result.insertions(),
        result.deletions(),
        result.errors(),
        result.wer()
    );
    eprintln!("\nOps:");

    fn render(c: &Canonical) -> String {
        match c {
            Canonical::Word(s) => format!("\"{}\"", s),
            Canonical::Number(n) => format!("#{}", n),
        }
    }

    for op in &result.ops {
        match op {
            Op::Match {
                ref_range,
                hyp_range,
            } => {
                let rs: Vec<String> = ref_range
                    .clone()
                    .map(|i| render(&result.ref_tokens[i].canonical))
                    .collect();
                let hs: Vec<String> = hyp_range
                    .clone()
                    .map(|i| render(&result.hyp_tokens[i].canonical))
                    .collect();
                eprintln!("  =  {} ≡ {}", rs.join(" "), hs.join(" "));
            }
            Op::Sub { ref_idx, hyp_idx } => {
                eprintln!(
                    "  S  {}  →  {}",
                    render(&result.ref_tokens[*ref_idx].canonical),
                    render(&result.hyp_tokens[*hyp_idx].canonical)
                );
            }
            Op::Ins { hyp_idx } => {
                eprintln!("  +  {}", render(&result.hyp_tokens[*hyp_idx].canonical));
            }
            Op::Del { ref_idx } => {
                eprintln!("  -  {}", render(&result.ref_tokens[*ref_idx].canonical));
            }
        }
    }
}

#[test]
fn op_timing_computes_start_offset_for_framework() {
    // Demonstrate the kind of metric a framework would compute.
    let r = vec![tw("hello", 0.0, 0.5), tw("world", 0.5, 1.0)];
    let h = vec![tw("hello", 0.05, 0.55), tw("world", 0.60, 1.10)];
    let result = compare(&r, &h, Language::English);

    let mut offsets = Vec::new();
    for op in &result.ops {
        let t = result.op_timing(op);
        if let (Some(r), Some(h)) = (t.ref_span, t.hyp_span) {
            offsets.push(h.start - r.start);
        }
    }
    assert_eq!(offsets.len(), 2);
    assert!((offsets[0] - 0.05).abs() < 1e-9);
    assert!((offsets[1] - 0.10).abs() < 1e-9);
}
