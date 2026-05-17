use std::ops::Range;

use crate::token::{Canonical, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Match { ref_range: Range<usize>, hyp_range: Range<usize> },
    Sub { ref_idx: usize, hyp_idx: usize },
    Ins { hyp_idx: usize },
    Del { ref_idx: usize },
}

const MERGE_CAP: usize = 3;

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

            if !canonicals_equal && should_try_merge(&ref_tokens[i - 1], &hyp_tokens[j - 1]) {
                let amax = MERGE_CAP.min(i);
                let bmax = MERGE_CAP.min(j);
                for a in 1..=amax {
                    for b in 1..=bmax {
                        if a == 1 && b == 1 {
                            continue;
                        }
                        if !all_words(&ref_tokens[i - a..i]) || !all_words(&hyp_tokens[j - b..j]) {
                            continue;
                        }
                        if concat_words(&ref_tokens[i - a..i])
                            == concat_words(&hyp_tokens[j - b..j])
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
                ops.push(Op::Sub { ref_idx: i - 1, hyp_idx: j - 1 });
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

fn should_try_merge(r: &Token, h: &Token) -> bool {
    let (rw, hw) = match (&r.canonical, &h.canonical) {
        (Canonical::Word(rw), Canonical::Word(hw)) => (rw, hw),
        _ => return false,
    };
    rw.ends_with(hw.as_str()) || hw.ends_with(rw.as_str())
}

fn all_words(toks: &[Token]) -> bool {
    toks.iter().all(|t| matches!(t.canonical, Canonical::Word(_)))
}

fn concat_words(toks: &[Token]) -> String {
    let mut out = String::new();
    for t in toks {
        if let Canonical::Word(s) = &t.canonical {
            out.push_str(s);
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
            Op::Match { ref_range, hyp_range } => {
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
            Op::Match { ref_range, hyp_range } => {
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
    fn number_token_does_not_merge_with_word() {
        let r = vec![num_tok(2), word_tok("president")];
        let h = vec![word_tok("2president")];
        let ops = align(&r, &h);
        // Should NOT fold into one Match — numbers don't merge with words.
        assert!(!ops.iter().all(|o| matches!(o, Op::Match { .. })));
    }
}
