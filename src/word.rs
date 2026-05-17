#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub text: String,
    pub start: f64,
    pub end: f64,
}

impl Word {
    pub fn new(text: impl Into<String>, start: f64, end: f64) -> Self {
        Self {
            text: text.into(),
            start,
            end,
        }
    }
}
