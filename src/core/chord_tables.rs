//! Forte set-class (pitch-class set) reference tables
//!
//! Covers trichords (3-note sets, Forte 3-1 through 3-12) and tetrachords
//! (4-note sets, Forte 4-1 through 4-29) — the two cardinalities most
//! relevant to everyday tonal/set-theory chord analysis (triads and
//! seventh chords). Pentachords and larger (5-x through 9-x, roughly 160
//! further entries) are deliberately not included: with no reliable way
//! to verify a hand-transcribed table of that size in this environment,
//! it's safer to leave them unrecognized (`None`) than to risk silently
//! wrong Forte numbers for real analysis work.
//!
//! Data verified against a standard "List of pitch-class sets" reference
//! (prime forms + Forte numbers for cardinalities 3 and 4, including the
//! 4-Z15/4-Z29 Z-related pair, the only Z-relation at these cardinalities).

/// (prime form pitch classes, Forte number, Z-relation partner if any)
type ForteEntry = (&'static [u8], &'static str, Option<&'static str>);

const TRICHORDS: [ForteEntry; 12] = [
    (&[0, 1, 2], "3-1", None),
    (&[0, 1, 3], "3-2", None),
    (&[0, 1, 4], "3-3", None),
    (&[0, 1, 5], "3-4", None),
    (&[0, 1, 6], "3-5", None),
    (&[0, 2, 4], "3-6", None),
    (&[0, 2, 5], "3-7", None),
    (&[0, 2, 6], "3-8", None),
    (&[0, 2, 7], "3-9", None),
    (&[0, 3, 6], "3-10", None),
    (&[0, 3, 7], "3-11", None),
    (&[0, 4, 8], "3-12", None),
];

const TETRACHORDS: [ForteEntry; 29] = [
    (&[0, 1, 2, 3], "4-1", None),
    (&[0, 1, 2, 4], "4-2", None),
    (&[0, 1, 3, 4], "4-3", None),
    (&[0, 1, 2, 5], "4-4", None),
    (&[0, 1, 2, 6], "4-5", None),
    (&[0, 1, 2, 7], "4-6", None),
    (&[0, 1, 4, 5], "4-7", None),
    (&[0, 1, 5, 6], "4-8", None),
    (&[0, 1, 6, 7], "4-9", None),
    (&[0, 2, 3, 5], "4-10", None),
    (&[0, 1, 3, 5], "4-11", None),
    (&[0, 2, 3, 6], "4-12", None),
    (&[0, 1, 3, 6], "4-13", None),
    (&[0, 2, 3, 7], "4-14", None),
    (&[0, 1, 4, 6], "4-Z15", Some("4-Z29")),
    (&[0, 1, 5, 7], "4-16", None),
    (&[0, 3, 4, 7], "4-17", None),
    (&[0, 1, 4, 7], "4-18", None),
    (&[0, 1, 4, 8], "4-19", None),
    (&[0, 1, 5, 8], "4-20", None),
    (&[0, 2, 4, 6], "4-21", None),
    (&[0, 2, 4, 7], "4-22", None),
    (&[0, 2, 5, 7], "4-23", None),
    (&[0, 2, 4, 8], "4-24", None),
    (&[0, 2, 6, 8], "4-25", None),
    (&[0, 3, 5, 8], "4-26", None),
    (&[0, 2, 5, 8], "4-27", None),
    (&[0, 3, 6, 9], "4-28", None),
    (&[0, 1, 3, 7], "4-Z29", Some("4-Z15")),
];

/// Look up the Forte set-class number (and Z-relation partner, if any) for
/// a prime-form pitch-class set (see `Chord::prime_form`). Only trichords
/// and tetrachords are covered (see module docs); returns `None` for other
/// cardinalities or a prime form that doesn't match one of the listed
/// canonical forms (which can happen if this crate's own compactness
/// tie-break for `prime_form` picks a different representative than
/// Forte's canonical listing for an asymmetric set).
pub fn forte_class_for_prime_form(
    prime_form: &[u8],
) -> Option<(&'static str, Option<&'static str>)> {
    let table: &[ForteEntry] = match prime_form.len() {
        3 => &TRICHORDS,
        4 => &TETRACHORDS,
        _ => return None,
    };
    table
        .iter()
        .find(|(pf, _, _)| *pf == prime_form)
        .map(|(_, name, z)| (*name, *z))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_major_triad_forte_class() {
        assert_eq!(forte_class_for_prime_form(&[0, 3, 7]), Some(("3-11", None)));
    }

    #[test]
    fn test_diminished_triad_forte_class() {
        assert_eq!(forte_class_for_prime_form(&[0, 3, 6]), Some(("3-10", None)));
    }

    #[test]
    fn test_z_related_tetrachords() {
        assert_eq!(
            forte_class_for_prime_form(&[0, 1, 4, 6]),
            Some(("4-Z15", Some("4-Z29")))
        );
        assert_eq!(
            forte_class_for_prime_form(&[0, 1, 3, 7]),
            Some(("4-Z29", Some("4-Z15")))
        );
    }

    #[test]
    fn test_unsupported_cardinality_returns_none() {
        assert_eq!(forte_class_for_prime_form(&[0, 2, 4, 6, 8]), None);
    }
}
