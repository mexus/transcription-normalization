#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub word_indices: Vec<usize>,
    pub raw: String,
    pub canonical: Canonical,
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
