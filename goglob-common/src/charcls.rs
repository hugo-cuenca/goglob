mod sealed {
    #![allow(non_camel_case_types)]

    use std::cmp::Ordering;
    use std::fmt::{self, Formatter, Write};
    use std::hash::{Hash, Hasher};
    use std::ops::RangeInclusive;

    #[derive(Default, Copy, Clone, Eq)]
    #[repr(transparent)]
    pub struct char_sealed(pub char);
    impl Ord for char_sealed {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0.cmp(&other.0)
        }

        fn max(self, other: Self) -> Self
        where
            Self: Sized,
        {
            char_sealed(self.0.max(other.0))
        }

        fn min(self, other: Self) -> Self
        where
            Self: Sized,
        {
            char_sealed(self.0.min(other.0))
        }

        fn clamp(self, min: Self, max: Self) -> Self
        where
            Self: Sized,
        {
            char_sealed(self.0.clamp(min.0, max.0))
        }
    }
    impl PartialOrd<char_sealed> for char_sealed {
        fn partial_cmp(&self, other: &char_sealed) -> Option<Ordering> {
            self.0.partial_cmp(&other.0)
        }

        fn lt(&self, other: &char_sealed) -> bool {
            self.0.lt(&other.0)
        }

        fn le(&self, other: &char_sealed) -> bool {
            self.0.le(&other.0)
        }

        fn gt(&self, other: &char_sealed) -> bool {
            self.0.gt(&other.0)
        }

        fn ge(&self, other: &char_sealed) -> bool {
            self.0.ge(&other.0)
        }
    }
    impl PartialOrd<char> for char_sealed {
        fn partial_cmp(&self, other: &char) -> Option<Ordering> {
            self.0.partial_cmp(other)
        }

        fn lt(&self, other: &char) -> bool {
            self.0.lt(other)
        }

        fn le(&self, other: &char) -> bool {
            self.0.le(other)
        }

        fn gt(&self, other: &char) -> bool {
            self.0.gt(other)
        }

        fn ge(&self, other: &char) -> bool {
            self.0.ge(other)
        }
    }
    impl PartialEq<char_sealed> for char_sealed {
        fn eq(&self, other: &char_sealed) -> bool {
            (&self.0).eq(&other.0)
        }
    }
    impl PartialEq<char> for char_sealed {
        fn eq(&self, other: &char) -> bool {
            (&self.0).eq(other)
        }
    }
    impl Hash for char_sealed {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state)
        }

        fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
        where
            Self: Sized,
        {
            // SAFETY: char_sealed has same representation as char
            char::hash_slice(unsafe { &*(data as *const _ as *const _) }, state)
        }
    }
    impl fmt::Debug for char_sealed {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_char(self.0)
        }
    }
    impl fmt::Display for char_sealed {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.write_char(self.0)
        }
    }

    #[derive(Clone, Eq)]
    #[repr(transparent)]
    pub struct RangeInclusive_char_sealed(pub RangeInclusive<char>);
    impl PartialEq<RangeInclusive_char_sealed> for RangeInclusive_char_sealed {
        fn eq(&self, other: &RangeInclusive_char_sealed) -> bool {
            (&self.0).eq(&other.0)
        }
    }
    impl PartialEq<RangeInclusive<char>> for RangeInclusive_char_sealed {
        fn eq(&self, other: &RangeInclusive<char>) -> bool {
            (&self.0).eq(other)
        }
    }
    impl Hash for RangeInclusive_char_sealed {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.0.hash(state)
        }

        fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
        where
            Self: Sized,
        {
            // SAFETY: RangeInclusive_char_sealed has same representation as RangeInclusive::<char>
            RangeInclusive::<char>::hash_slice(unsafe { &*(data as *const _ as *const _) }, state)
        }
    }
    impl fmt::Debug for RangeInclusive_char_sealed {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            self.0.fmt(f)
        }
    }
}

use crate::charcls::sealed::{char_sealed, RangeInclusive_char_sealed};
use core::ops::RangeInclusive;
use std::borrow::Cow;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CharClass {
    negated: bool,
    matches: Cow<'static, [CharClassType]>,
}
impl CharClass {
    pub fn new(negated: bool, mut matches: Vec<CharClassType>) -> Self {
        matches.shrink_to_fit();
        Self {
            negated,
            matches: Cow::Owned(matches),
        }
    }

    pub fn is_negated(&self) -> bool {
        self.negated
    }

    pub fn matches_next<'a>(&self, name: &'a str) -> Option<&'a str> {
        name.strip_prefix(|c| self.matches_char(c))
    }
    fn matches_char(&self, character: char) -> bool {
        self.matches.iter().any(|cct| cct.matches(character)) != self.negated
    }
}
impl IntoIterator for CharClass {
    type Item = CharClassType;
    type IntoIter = std::vec::IntoIter<CharClassType>;

    // `into_owned()` is necessary as the underlying object is dropped.
    // CharClass' `into_iter(self)` is only used internally by
    // `goglob-proc-macro`, meaning these heap allocations only happen
    // at compile-time anyway, so it shouldn't be of much concern.
    #[allow(clippy::unnecessary_to_owned)]
    fn into_iter(self) -> Self::IntoIter {
        self.matches.into_owned().into_iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CharClassType {
    Single(sealed::char_sealed),
    Range(sealed::RangeInclusive_char_sealed),
}
impl CharClassType {
    fn matches(&self, character: char) -> bool {
        match self {
            CharClassType::Single(sealed::char_sealed(char_match)) => *char_match == character,
            CharClassType::Range(sealed::RangeInclusive_char_sealed(range_match)) => {
                range_match.contains(&character)
            }
        }
    }
}
impl From<char> for CharClassType {
    fn from(char_match: char) -> Self {
        Self::Single(sealed::char_sealed(char_match))
    }
}
impl TryFrom<RangeInclusive<char>> for CharClassType {
    type Error = ();

    fn try_from(range_match: RangeInclusive<char>) -> Result<Self, Self::Error> {
        (!range_match.is_empty())
            .then(|| range_match)
            .map(sealed::RangeInclusive_char_sealed)
            .map(Self::Range)
            .ok_or(())
    }
}
impl From<char_sealed> for char {
    fn from(c: char_sealed) -> Self {
        c.0
    }
}
impl From<RangeInclusive_char_sealed> for RangeInclusive<char> {
    fn from(rg: RangeInclusive_char_sealed) -> Self {
        rg.0
    }
}

/// Internal workspace-only function employed by `goglob-proc-macro`.
///
/// The procedural macro will insert calls to this function in the end-user's project,
/// so it must be declared public.
pub const fn from_static(negated: bool, matches: &'static [CharClassType]) -> CharClass {
    CharClass {
        negated,
        matches: Cow::Borrowed(matches),
    }
}

/// Internal workspace-only function employed by `goglob-proc-macro`.
///
/// The procedural macro will insert calls to this function in the end-user's project,
/// so it must be declared public.
pub const fn type_from_char(c: char) -> CharClassType {
    CharClassType::Single(sealed::char_sealed(c))
}

/// Internal workspace-only function employed by `goglob-proc-macro`.
///
/// The procedural macro will insert calls to this function in the end-user's project,
/// so it must be declared public.
///
/// All safety guarantees are upheld by the procedural macro.
///
/// # Safety
/// Range must not be empty.
pub const unsafe fn type_from_range_unchecked(r: RangeInclusive<char>) -> CharClassType {
    CharClassType::Range(sealed::RangeInclusive_char_sealed(r))
}

#[cfg(test)]
mod tests {
    use crate::charcls::{self, CharClass, CharClassType};

    #[test]
    fn charclass_matches_next() {
        let class = CharClass::new(
            false,
            vec!['a'.into(), 'b'.into(), ('c'..='e').try_into().unwrap()],
        );
        assert_eq!(class.matches_next("abcdef"), Some("bcdef"));
        assert_eq!(class.matches_next("bcdefa"), Some("cdefa"));
        assert_eq!(class.matches_next("cdefab"), Some("defab"));
        assert_eq!(class.matches_next("defabc"), Some("efabc"));
        assert_eq!(class.matches_next("efabcd"), Some("fabcd"));
        assert_eq!(class.matches_next("fabcde"), None);
        assert_eq!(class.matches_next("a"), Some(""));

        let class = CharClass::new(
            true,
            vec!['a'.into(), 'b'.into(), ('c'..='e').try_into().unwrap()],
        );
        assert_eq!(class.matches_next("abcdef"), None);
        assert_eq!(class.matches_next("bcdefa"), None);
        assert_eq!(class.matches_next("cdefab"), None);
        assert_eq!(class.matches_next("defabc"), None);
        assert_eq!(class.matches_next("efabcd"), None);
        assert_eq!(class.matches_next("fabcde"), Some("abcde"));
        assert_eq!(class.matches_next("f"), Some(""));
    }

    #[test]
    fn charclass_matches_next_static() {
        // SAFETY: The range 'c'..='e' is not empty.
        static TYPE_TOKENS: &[CharClassType] = &[
            charcls::type_from_char('a'),
            charcls::type_from_char('b'),
            unsafe { charcls::type_from_range_unchecked('c'..='e') },
        ];

        let class = charcls::from_static(false, TYPE_TOKENS);
        assert_eq!(class.matches_next("abcdef"), Some("bcdef"));
        assert_eq!(class.matches_next("bcdefa"), Some("cdefa"));
        assert_eq!(class.matches_next("cdefab"), Some("defab"));
        assert_eq!(class.matches_next("defabc"), Some("efabc"));
        assert_eq!(class.matches_next("efabcd"), Some("fabcd"));
        assert_eq!(class.matches_next("fabcde"), None);
        assert_eq!(class.matches_next("a"), Some(""));

        let class = charcls::from_static(true, TYPE_TOKENS);
        assert_eq!(class.matches_next("abcdef"), None);
        assert_eq!(class.matches_next("bcdefa"), None);
        assert_eq!(class.matches_next("cdefab"), None);
        assert_eq!(class.matches_next("defabc"), None);
        assert_eq!(class.matches_next("efabcd"), None);
        assert_eq!(class.matches_next("fabcde"), Some("abcde"));
        assert_eq!(class.matches_next("f"), Some(""));
    }

    #[test]
    fn charclasstype_conversion() {
        let class_type: CharClassType = 'a'.into();
        assert!(class_type.matches('a'));
        assert!(!class_type.matches('b'));

        let class_type: CharClassType = ('a'..='e').try_into().unwrap();
        assert!(class_type.matches('a'));
        assert!(class_type.matches('b'));
        assert!(class_type.matches('c'));
        assert!(class_type.matches('d'));
        assert!(class_type.matches('e'));
        assert!(!class_type.matches('f'));

        let class_type: CharClassType = ('a'..='a').try_into().unwrap();
        assert!(class_type.matches('a'));
        assert!(!class_type.matches('b'));

        let class_type: Result<CharClassType, ()> = ('e'..='a').try_into();
        let _ = class_type.err().unwrap();
    }
}
