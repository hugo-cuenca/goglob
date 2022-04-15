use std::borrow::{Borrow, Cow};

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Literal(Cow<'static, str>);
impl Literal {
    pub fn new(literal: String) -> Self {
        Self(Cow::Owned(literal))
    }

    pub(crate) fn matches_next<'a>(&self, name: &'a str) -> Option<&'a str> {
        name.strip_prefix(self.0.as_ref())
    }
}
impl AsRef<str> for Literal {
    fn as_ref(&self) -> &str {
        self.0.borrow()
    }
}

/// Internal workspace-only function employed by `goglob-proc-macro`.
///
/// The procedural macro will insert calls to this function in the end-user's project,
/// so it must be declared public.
pub const fn from_static(literal: &'static str) -> Literal {
    Literal(Cow::Borrowed(literal))
}

#[cfg(test)]
mod tests {
    use crate::literal::{self, Literal};

    #[test]
    fn literal_matches_next() {
        let literal: Literal = Literal::new("abcde".into());
        assert_eq!(literal.matches_next("abcdefg"), Some("fg"));
        assert_eq!(literal.matches_next("fgabcde"), None);
        assert_eq!(literal.matches_next("abceefg"), None);
        assert_eq!(literal.matches_next("abcd"), None);
        assert_eq!(literal.matches_next("abcde"), Some(""));
    }

    #[test]
    fn literal_matches_next_static() {
        let literal: Literal = literal::from_static("abcde");
        assert_eq!(literal.matches_next("abcdefg"), Some("fg"));
        assert_eq!(literal.matches_next("fgabcde"), None);
        assert_eq!(literal.matches_next("abceefg"), None);
        assert_eq!(literal.matches_next("abcd"), None);
        assert_eq!(literal.matches_next("abcde"), Some(""));
    }
}
