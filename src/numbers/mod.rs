use crate::language::Language;
use crate::token::Fragment;

pub(crate) mod english;
pub(crate) mod russian;

pub(crate) fn try_consume(lang: Language, frags: &[Fragment]) -> Option<(usize, i64)> {
    match lang {
        Language::English => english::try_consume(frags),
        Language::Russian => russian::try_consume(frags),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Atom {
    Unit(i64),
    Teen(i64),
    Tens(i64),
    Hundreds(i64),
    HundredScale,
    Scale(i64),
    Glue,
    Zero,
}

pub(crate) fn grammar_parse(atoms: &[Atom]) -> Option<i64> {
    if atoms.is_empty() {
        return None;
    }
    if atoms.len() == 1 && matches!(atoms[0], Atom::Zero) {
        return Some(0);
    }

    let mut total: i64 = 0;
    let mut segment: i64 = 0;

    let mut saw_units = false;
    let mut saw_teen = false;
    let mut saw_tens = false;
    let mut saw_hundreds = false;

    for atom in atoms {
        match *atom {
            Atom::Zero => return None,
            Atom::Unit(n) => {
                if saw_units || saw_teen {
                    return None;
                }
                segment = segment.checked_add(n)?;
                saw_units = true;
            }
            Atom::Teen(n) => {
                if saw_units || saw_teen || saw_tens {
                    return None;
                }
                segment = segment.checked_add(n)?;
                saw_teen = true;
            }
            Atom::Tens(n) => {
                if saw_units || saw_teen || saw_tens {
                    return None;
                }
                segment = segment.checked_add(n)?;
                saw_tens = true;
            }
            Atom::Hundreds(n) => {
                if saw_hundreds || saw_tens || saw_teen || saw_units {
                    return None;
                }
                segment = segment.checked_add(n)?;
                saw_hundreds = true;
            }
            Atom::HundredScale => {
                if saw_hundreds {
                    return None;
                }
                let mult = if segment == 0 {
                    1
                } else if segment <= 99 {
                    segment
                } else {
                    return None;
                };
                segment = mult * 100;
                saw_units = false;
                saw_teen = false;
                saw_tens = false;
                saw_hundreds = true;
            }
            Atom::Scale(v) => {
                let block = if segment == 0 && !saw_units && !saw_teen && !saw_tens && !saw_hundreds {
                    1
                } else {
                    segment
                };
                let chunk = block.checked_mul(v)?;
                total = total.checked_add(chunk)?;
                segment = 0;
                saw_units = false;
                saw_teen = false;
                saw_tens = false;
                saw_hundreds = false;
            }
            Atom::Glue => {}
        }
    }

    total.checked_add(segment)
}
