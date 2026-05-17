#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub word_indices: Vec<usize>,
    pub raw: String,
    pub canonical: Canonical,
    /// Min `start` over all source `Word`s this token spans.
    pub start: f64,
    /// Max `end` over all source `Word`s this token spans.
    pub end: f64,
}

impl Token {
    pub fn span(&self) -> TimeSpan {
        TimeSpan { start: self.start, end: self.end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeSpan {
    pub start: f64,
    pub end: f64,
}

impl TimeSpan {
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Canonical {
    Word(String),
    Number(i64),
}

impl Canonical {
    pub fn as_word(&self) -> Option<&str> {
        match self {
            Canonical::Word(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Fragment {
    pub word_index: usize,
    pub text: String,
}
