//! Transcription comparison library — fair WER between reference and hypothesis
//! token streams that tolerates numeral form (`100` ≡ `сто` ≡ `сотый` ≡ `one hundred`)
//! and in-word hyphen variants (`вице-президент` ≡ `вицепрезидент` ≡ `вице президент`).

mod align;
mod language;
mod normalize;
mod numbers;
mod result;
mod token;
mod word;

pub use align::Op;
pub use language::Language;
pub use result::{AlignmentResult, OpTiming};
pub use token::{Canonical, TimeSpan, Token};
pub use word::Word;

pub fn compare(reference: &[Word], hypothesis: &[Word], lang: Language) -> AlignmentResult {
    let ref_tokens = normalize::tokenize(reference, lang);
    let hyp_tokens = normalize::tokenize(hypothesis, lang);
    let ops = align::align(&ref_tokens, &hyp_tokens);
    AlignmentResult {
        ref_tokens,
        hyp_tokens,
        ops,
    }
}
