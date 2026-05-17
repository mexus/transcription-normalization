# transcription-normalization

Fair WER between a reference transcription and a transcriber's output, without
penalizing **numeral form** or **in-word hyphen variants**.

```text
"100"              ≡ "сто"           ≡ "сотый"
"one hundred"      ≡ "hundredth"     ≡ "100"
"вице-президент"   ≡ "вицепрезидент" ≡ "вице президент"
"first of January" ≡ "1 of January"
```

Languages: **Russian** and **English**. No external dependencies.

## Usage

```rust
use transcription_normalization::{compare, Language, Word};

let reference = vec![
    Word::new("сто",      0.0, 0.3),
    Word::new("двадцать", 0.3, 0.6),
    Word::new("три",      0.6, 0.8),
    Word::new("рубля",    0.8, 1.1),
];
let hypothesis = vec![
    Word::new("123",   0.0, 0.8),
    Word::new("рубля", 0.8, 1.1),
];

let result = compare(&reference, &hypothesis, Language::Russian);
assert_eq!(result.errors(), 0);
assert_eq!(result.wer(),    0.0);
```

`Word` is `{ text: String, start: f64, end: f64 }`. Timestamps are passed
through to the result so you can attribute errors to time spans, but the
comparison itself is purely textual.

## Aggregating across a dataset

`AlignmentResult` is designed for batch-style WER, not just per-pair:

```rust
let mut total_errors = 0usize;
let mut total_ref    = 0usize;

for (reference, hypothesis) in dataset {
    let r = compare(&reference, &hypothesis, Language::Russian);
    total_errors += r.errors();
    total_ref    += r.ref_token_count();
}

let dataset_wer = total_errors as f64 / total_ref as f64;
```

Per-pair you also get `substitutions()`, `insertions()`, `deletions()`, and
`ops: Vec<Op>` — each op carries token-index ranges into `ref_tokens` /
`hyp_tokens`, and each token carries `word_indices` back to the source `Word`s.

## What it does and doesn't do

| | |
|---|---|
| Strips internal hyphens when comparing | ✅ |
| Folds Russian `ё → е` | ✅ |
| Collapses `"сто двадцать три"` and `"123"` to the same token | ✅ |
| Ignores cardinal vs ordinal vs case for numerals | ✅ |
| Mixed-language input | ❌ |
| Decimal numbers, percentages, `1990s`, `100k` | ❌ (left as words) |
| Partial credit (cardinal ≈ ordinal as ½-error) | ❌ (strict equality) |

See `tests/integration.rs` for more realistic examples.
