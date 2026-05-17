use crate::align::Op;
use crate::token::Token;

#[derive(Debug, Clone)]
pub struct AlignmentResult {
    pub ref_tokens: Vec<Token>,
    pub hyp_tokens: Vec<Token>,
    pub ops: Vec<Op>,
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
}
