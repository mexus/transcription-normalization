# transcription-normalization

Fair WER between a reference transcription and a transcriber's output, without
penalizing **numeral form** or **in-word hyphen variants** — with per-op
timing so frameworks can measure timing errors alongside word errors.

```text
"100"              ≡ "сто"           ≡ "сотый"
"one hundred"      ≡ "hundredth"     ≡ "100"
"вице-президент"   ≡ "вицепрезидент" ≡ "вице президент"
"first of January" ≡ "1 of January"
```

Languages: **Russian** and **English**. No external dependencies.

## Per-pair usage

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

`Word` is `{ text: String, start: f64, end: f64 }`. Timestamps flow through to
the result; the comparison itself is purely textual.

## Timing errors

Each `Op` in `result.ops` is one of `Match`, `Sub`, `Ins`, `Del`. To get the
time spans on each side, call `result.op_timing(&op)`:

```rust
use transcription_normalization::{compare, Language, Word};

let result = compare(&reference, &hypothesis, Language::Russian);

for op in &result.ops {
    let t = result.op_timing(op);
    match (t.ref_span, t.hyp_span) {
        (Some(r), Some(h)) => {
            let start_offset    = h.start - r.start;
            let end_offset      = h.end   - r.end;
            let duration_error  = h.duration() - r.duration();
            // ... feed into framework metrics
        }
        (Some(r), None) => { /* deletion at r */ }
        (None, Some(h)) => { /* insertion at h */ }
        (None, None)    => unreachable!(),
    }
}
```

**The N-M Match case is handled internally.** When the aligner merges multiple
tokens on one side to match a single token on the other (e.g.
`["вице", "президент"]` ↔ `["вицепрезидент"]`), `op_timing` returns one span
per side — the **union** of every token in the matched range. Callers do not
need to differentiate between 1-1 and N-M matches:

```rust
// ref: one Word "вице-президент" with span 0.0..1.0
// hyp: two Words "вице" 0.2..0.6 and "президент" 0.6..1.1
// → one Op::Match with ref_range 0..1, hyp_range 0..2
let t = result.op_timing(&result.ops[0]);
assert_eq!(t.ref_span.unwrap().start, 0.0);
assert_eq!(t.ref_span.unwrap().end,   1.0);
assert_eq!(t.hyp_span.unwrap().start, 0.2);
assert_eq!(t.hyp_span.unwrap().end,   1.1);
```

The same applies when a number run like `"сто двадцать три"` (3 input `Word`s)
collapses into a single `Number(123)` token — its span is the union over all 3.

## Dataset-level aggregation

`AlignmentResult` is designed for batch WER, not just single-pair:

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

For timing metrics, accumulate offsets/IoUs across all `op_timing` calls and
average however your framework defines it.

## What you get back

`AlignmentResult` exposes:

| Field / method                | Type / meaning                                                   |
|-------------------------------|------------------------------------------------------------------|
| `ref_tokens`, `hyp_tokens`    | `Vec<Token>` — canonical token streams after normalization       |
| `ops`                         | `Vec<Op>` — alignment ops with token-index ranges                |
| `substitutions()` / `insertions()` / `deletions()` | per-op counts                               |
| `errors()`                    | S + I + D                                                        |
| `ref_token_count()`           | denominator for WER                                              |
| `wer()`                       | `errors() / ref_token_count()`                                   |
| `op_timing(&op)`              | `OpTiming { ref_span, hyp_span }` — see above                    |

`Token` exposes:

| Field          | Meaning                                                                |
|----------------|------------------------------------------------------------------------|
| `word_indices` | indices into the source `&[Word]` slice this token spans                |
| `raw`          | normalized text (lowercased, `ё→е`, punct stripped, hyphens kept)       |
| `canonical`    | `Word(String)` (hyphen-stripped) or `Number(i64)` — the comparison key  |
| `start`, `end` | min/max time bounds over the source `Word`s                             |
| `.span()`      | `TimeSpan { start, end }` convenience                                   |

## What it does and doesn't do

| | |
|---|---|
| Strips internal hyphens when comparing | ✅ |
| Folds Russian `ё → е` | ✅ |
| Collapses `"сто двадцать три"` and `"123"` to the same token | ✅ |
| Ignores cardinal vs ordinal vs case for numerals | ✅ |
| Exposes union-of-range time spans for N-M matches | ✅ |
| Mixed-language input | ❌ |
| Decimal numbers, percentages, `1990s`, `100k` | ❌ (left as words) |
| Partial credit (cardinal ≈ ordinal as ½-error) | ❌ (strict equality) |

See `tests/integration.rs` for more realistic examples and `CLAUDE.md` for
architecture notes.
