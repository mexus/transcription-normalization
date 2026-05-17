use std::ops::Range;

use crate::token::{Canonical, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Match {
        ref_range: Range<usize>,
        hyp_range: Range<usize>,
    },
    Sub {
        ref_idx: usize,
        hyp_idx: usize,
    },
    Ins {
        hyp_idx: usize,
    },
    Del {
        ref_idx: usize,
    },
}

const MERGE_CAP: usize = 3;
/// Numbers can split or join across many tokens (ASR may emit a phone number
/// as one digit run while a human spells out each digit), so allow a deeper
/// merge depth when both sides of the merge are pure number runs.
const MERGE_NUM_CAP: usize = 16;

#[derive(Debug, Clone, Copy)]
enum Back {
    Start,
    Del,
    Ins,
    Sub,
    Match { a: usize, b: usize },
}

pub(crate) fn align(ref_tokens: &[Token], hyp_tokens: &[Token]) -> Vec<Op> {
    let n = ref_tokens.len();
    let m = hyp_tokens.len();

    let mut d = vec![vec![0usize; m + 1]; n + 1];
    let mut back = vec![vec![Back::Start; m + 1]; n + 1];

    for i in 1..=n {
        d[i][0] = i;
        back[i][0] = Back::Del;
    }
    for j in 1..=m {
        d[0][j] = j;
        back[0][j] = Back::Ins;
    }

    for i in 1..=n {
        for j in 1..=m {
            let del_cost = d[i - 1][j] + 1;
            let ins_cost = d[i][j - 1] + 1;
            let canonicals_equal = ref_tokens[i - 1].canonical == hyp_tokens[j - 1].canonical;
            let sub_cost = d[i - 1][j - 1] + if canonicals_equal { 0 } else { 1 };

            let mut best = del_cost;
            let mut tr = Back::Del;

            if ins_cost < best {
                best = ins_cost;
                tr = Back::Ins;
            }
            if sub_cost < best {
                best = sub_cost;
                tr = if canonicals_equal {
                    Back::Match { a: 1, b: 1 }
                } else {
                    Back::Sub
                };
            } else if sub_cost == best && canonicals_equal {
                // Prefer exact match over equal-cost del/ins so backtrack produces a Match op.
                tr = Back::Match { a: 1, b: 1 };
            }

            if !canonicals_equal
                && let Some(kind) = merge_kind(&ref_tokens[i - 1], &hyp_tokens[j - 1])
            {
                let cap = match kind {
                    MergeKind::Word => MERGE_CAP,
                    MergeKind::Number => MERGE_NUM_CAP,
                };
                let amax = cap.min(i);
                let bmax = cap.min(j);
                for a in 1..=amax {
                    for b in 1..=bmax {
                        if a == 1 && b == 1 {
                            continue;
                        }
                        if !all_of_kind(&ref_tokens[i - a..i], kind)
                            || !all_of_kind(&hyp_tokens[j - b..j], kind)
                        {
                            continue;
                        }
                        if concat_kind(&ref_tokens[i - a..i], kind)
                            == concat_kind(&hyp_tokens[j - b..j], kind)
                        {
                            let cost = d[i - a][j - b];
                            if cost < best {
                                best = cost;
                                tr = Back::Match { a, b };
                            }
                        }
                    }
                }
            }

            d[i][j] = best;
            back[i][j] = tr;
        }
    }

    let mut ops = Vec::new();
    let mut i = n;
    let mut j = m;
    while i > 0 || j > 0 {
        match back[i][j] {
            Back::Start => break,
            Back::Del => {
                ops.push(Op::Del { ref_idx: i - 1 });
                i -= 1;
            }
            Back::Ins => {
                ops.push(Op::Ins { hyp_idx: j - 1 });
                j -= 1;
            }
            Back::Sub => {
                ops.push(Op::Sub {
                    ref_idx: i - 1,
                    hyp_idx: j - 1,
                });
                i -= 1;
                j -= 1;
            }
            Back::Match { a, b } => {
                ops.push(Op::Match {
                    ref_range: (i - a)..i,
                    hyp_range: (j - b)..j,
                });
                i -= a;
                j -= b;
            }
        }
    }
    ops.reverse();
    ops
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MergeKind {
    Word,
    Number,
}

/// Returns the merge kind if the last two tokens are the same `Canonical` variant
/// AND one side's surface form (canonical word, or decimal string of a number) is
/// a suffix of the other. The suffix gate keeps the inner DP loop cheap: a valid
/// concat-equality merge must align character-by-character at the right end of
/// the merged string, so no suffix overlap means no possible merge.
fn merge_kind(r: &Token, h: &Token) -> Option<MergeKind> {
    match (&r.canonical, &h.canonical) {
        (Canonical::Word(rw), Canonical::Word(hw)) => {
            if rw.ends_with(hw.as_str()) || hw.ends_with(rw.as_str()) {
                Some(MergeKind::Word)
            } else {
                None
            }
        }
        (Canonical::Number(rn), Canonical::Number(hn)) => {
            let rs = rn.to_string();
            let hs = hn.to_string();
            if rs.ends_with(&hs) || hs.ends_with(&rs) {
                Some(MergeKind::Number)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn all_of_kind(toks: &[Token], kind: MergeKind) -> bool {
    toks.iter().all(|t| {
        matches!(
            (kind, &t.canonical),
            (MergeKind::Word, Canonical::Word(_)) | (MergeKind::Number, Canonical::Number(_))
        )
    })
}

fn concat_kind(toks: &[Token], kind: MergeKind) -> String {
    let mut out = String::new();
    for t in toks {
        match (kind, &t.canonical) {
            (MergeKind::Word, Canonical::Word(s)) => out.push_str(s),
            (MergeKind::Number, Canonical::Number(n)) => {
                use std::fmt::Write;
                let _ = write!(out, "{n}");
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word_tok(s: &str) -> Token {
        Token {
            word_indices: vec![0],
            raw: s.to_string(),
            canonical: Canonical::Word(s.replace('-', "")),
            start: 0.0,
            end: 0.0,
        }
    }

    fn num_tok(n: i64) -> Token {
        Token {
            word_indices: vec![0],
            raw: n.to_string(),
            canonical: Canonical::Number(n),
            start: 0.0,
            end: 0.0,
        }
    }

    #[test]
    fn empty_inputs() {
        assert!(align(&[], &[]).is_empty());
    }

    #[test]
    fn perfect_match() {
        let r = vec![word_tok("hello"), word_tok("world")];
        let h = r.clone();
        let ops = align(&r, &h);
        assert_eq!(ops.len(), 2);
        assert!(matches!(ops[0], Op::Match { .. }));
        assert!(matches!(ops[1], Op::Match { .. }));
    }

    #[test]
    fn single_substitution() {
        let r = vec![word_tok("hello"), word_tok("world")];
        let h = vec![word_tok("hello"), word_tok("there")];
        let ops = align(&r, &h);
        assert!(ops.iter().any(|o| matches!(o, Op::Sub { .. })));
    }

    #[test]
    fn insertion() {
        let r = vec![word_tok("hello")];
        let h = vec![word_tok("hello"), word_tok("world")];
        let ops = align(&r, &h);
        assert!(ops.iter().any(|o| matches!(o, Op::Ins { hyp_idx: 1 })));
    }

    #[test]
    fn deletion() {
        let r = vec![word_tok("hello"), word_tok("world")];
        let h = vec![word_tok("hello")];
        let ops = align(&r, &h);
        assert!(ops.iter().any(|o| matches!(o, Op::Del { ref_idx: 1 })));
    }

    #[test]
    fn hyphen_merge_split_vs_joined() {
        let r = vec![word_tok("вицепрезидент")];
        let h = vec![word_tok("вице"), word_tok("президент")];
        let ops = align(&r, &h);
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            Op::Match {
                ref_range,
                hyp_range,
            } => {
                assert_eq!(*ref_range, 0..1);
                assert_eq!(*hyp_range, 0..2);
            }
            _ => panic!("expected Match, got {:?}", ops[0]),
        }
    }

    #[test]
    fn hyphen_merge_joined_vs_split() {
        let r = vec![word_tok("вице"), word_tok("президент")];
        let h = vec![word_tok("вицепрезидент")];
        let ops = align(&r, &h);
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            Op::Match {
                ref_range,
                hyp_range,
            } => {
                assert_eq!(*ref_range, 0..2);
                assert_eq!(*hyp_range, 0..1);
            }
            _ => panic!("expected Match"),
        }
    }

    #[test]
    fn hyphen_inside_word_handled_by_canonical_strip() {
        // word_tok("вице-президент") yields Canonical::Word("вицепрезидент") via replace.
        let r = vec![word_tok("вице-президент")];
        let h = vec![word_tok("вицепрезидент")];
        let ops = align(&r, &h);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], Op::Match { .. }));
    }

    #[test]
    fn number_matches_regardless_of_form() {
        let r = vec![num_tok(123)];
        let h = vec![num_tok(123)];
        let ops = align(&r, &h);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], Op::Match { .. }));
    }

    #[test]
    fn suffix_gate_rejects_spurious_merge_context() {
        // "abc" vs "xyz" — no suffix overlap, no merge attempted. Verify we just get a sub.
        let r = vec![word_tok("abc")];
        let h = vec![word_tok("xyz")];
        let ops = align(&r, &h);
        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], Op::Sub { .. }));
    }

    #[test]
    fn number_run_merges_on_digit_concat() {
        // ref says "twenty zero zero", hyp says "2000": three numbers vs one,
        // decimal-string concat is "2000" on both sides.
        let r = vec![num_tok(20), num_tok(0), num_tok(0)];
        let h = vec![num_tok(2000)];
        let ops = align(&r, &h);
        assert_eq!(ops.len(), 1);
        match &ops[0] {
            Op::Match {
                ref_range,
                hyp_range,
            } => {
                assert_eq!(*ref_range, 0..3);
                assert_eq!(*hyp_range, 0..1);
            }
            other => panic!("expected Match, got {:?}", other),
        }
    }

    #[test]
    fn number_run_merges_phone_number_segmentation() {
        // ref enumerates digits, hyp groups them: both concatenate to "3709363".
        let r = vec![
            num_tok(3),
            num_tok(7),
            num_tok(0),
            num_tok(9),
            num_tok(3),
            num_tok(6),
            num_tok(3),
        ];
        let h = vec![num_tok(370), num_tok(9363)];
        let ops = align(&r, &h);
        // Should produce one or two Match ops covering everything (no Sub/Del/Ins).
        assert!(
            ops.iter().all(|o| matches!(o, Op::Match { .. })),
            "expected only Match ops, got {:?}",
            ops
        );
    }

    #[test]
    fn number_token_does_not_merge_with_word() {
        let r = vec![num_tok(2), word_tok("president")];
        let h = vec![word_tok("2president")];
        let ops = align(&r, &h);
        // Should NOT fold into one Match — numbers don't merge with words.
        assert!(!ops.iter().all(|o| matches!(o, Op::Match { .. })));
    }
}
