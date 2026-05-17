use std::ops::Range;

use crate::align::Op;
use crate::token::{TimeSpan, Token};

#[derive(Debug, Clone)]
pub struct AlignmentResult {
    pub ref_tokens: Vec<Token>,
    pub hyp_tokens: Vec<Token>,
    pub ops: Vec<Op>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpTiming {
    /// Reference-side time span. `None` for `Op::Ins` (no ref token).
    pub ref_span: Option<TimeSpan>,
    /// Hypothesis-side time span. `None` for `Op::Del` (no hyp token).
    pub hyp_span: Option<TimeSpan>,
}

impl AlignmentResult {
    pub fn substitutions(&self) -> usize {
        self.ops.iter().filter(|o| matches!(o, Op::Sub { .. })).count()
    }

    pub fn insertions(&self) -> usize {
        self.ops.iter().filter(|o| matches!(o, Op::Ins { .. })).count()
    }

    pub fn deletions(&self) -> usize {
        self.ops.iter().filter(|o| matches!(o, Op::Del { .. })).count()
    }

    pub fn ref_token_count(&self) -> usize {
        self.ref_tokens.len()
    }

    pub fn errors(&self) -> usize {
        self.substitutions() + self.insertions() + self.deletions()
    }

    pub fn wer(&self) -> f64 {
        let n = self.ref_token_count();
        if n == 0 {
            return if self.errors() == 0 { 0.0 } else { f64::INFINITY };
        }
        self.errors() as f64 / n as f64
    }

    /// Time spans on each side for an `Op`. For `Op::Match` with multi-token
    /// ranges (the hyphen-merge case), the returned span is the union of every
    /// token's span in the range — so callers don't need to differentiate
    /// between 1-1 and N-M matches when computing timing errors.
    pub fn op_timing(&self, op: &Op) -> OpTiming {
        match op {
            Op::Match { ref_range, hyp_range } => OpTiming {
                ref_span: span_over(&self.ref_tokens, ref_range.clone()),
                hyp_span: span_over(&self.hyp_tokens, hyp_range.clone()),
            },
            Op::Sub { ref_idx, hyp_idx } => OpTiming {
                ref_span: Some(self.ref_tokens[*ref_idx].span()),
                hyp_span: Some(self.hyp_tokens[*hyp_idx].span()),
            },
            Op::Ins { hyp_idx } => OpTiming {
                ref_span: None,
                hyp_span: Some(self.hyp_tokens[*hyp_idx].span()),
            },
            Op::Del { ref_idx } => OpTiming {
                ref_span: Some(self.ref_tokens[*ref_idx].span()),
                hyp_span: None,
            },
        }
    }
}

fn span_over(tokens: &[Token], range: Range<usize>) -> Option<TimeSpan> {
    if range.is_empty() {
        return None;
    }
    let slice = &tokens[range];
    let start = slice.iter().map(|t| t.start).fold(f64::INFINITY, f64::min);
    let end = slice.iter().map(|t| t.end).fold(f64::NEG_INFINITY, f64::max);
    Some(TimeSpan { start, end })
}
