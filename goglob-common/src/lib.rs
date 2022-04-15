//! # Do not use this library directly!
//!
//! See the `goglob` crate instead.

pub mod error;
pub use crate::error::Result;

pub mod charcls;
pub mod literal;

#[cfg(feature = "serde")]
mod serde;

use crate::{
    charcls::{CharClass as GlobTokenCharClass, CharClassType},
    error::{Error, ErrorType},
    literal::Literal as GlobTokenLiteral,
};
use std::{borrow::Cow, result::Result as StdResult};

/// Shell pattern matching similar to golang's `path.Match`.
///
/// # Further reading
///
/// See the `goglob` crate's documentation for the appropriate syntax.
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct GlobPattern {
    tokens: Cow<'static, [GlobToken]>,
}
impl GlobPattern {
    /// Compile the given `pattern` into tokens at runtime, returning a [`GlobPattern`][Self]
    /// on success or an [error][crate::error::Error] if `pattern` is syntactically invalid.
    ///
    /// # Further reading
    ///
    /// See the `goglob` crate's documentation for the appropriate syntax, as well as
    /// [goglob::error::Error][crate:error:Error] for possible syntax errors.
    #[inline]
    pub fn new<S: AsRef<str>>(pattern: S) -> Result<Self> {
        Self::_new(pattern.as_ref())
    }
    fn _new(pattern: &str) -> Result<Self> {
        let mut tokens = Vec::new();
        crate::scan_patterns(pattern, &mut tokens)?;

        tokens.shrink_to_fit();
        Ok(Self {
            tokens: Cow::Owned(tokens),
        })
    }

    /// Report whether the `name` matches the compiled shell pattern.
    ///
    /// # Further reading
    ///
    /// See the `goglob` crate's documentation for the appropriate syntax.
    #[inline]
    pub fn matches<S: AsRef<str>>(&self, name: S) -> bool {
        self._matches(name.as_ref())
    }
    fn _matches(&self, name: &str) -> bool {
        let mut next = name;
        let mut tokens = self.tokens.iter().peekable();
        'outer: while let Some(token) = tokens.next() {
            next = match token.try_matches_next(next) {
                Ok(Some(next)) => next,
                Ok(None) => return false,
                Err(()) => {
                    // SeqWildcard doesn't implement matches_next. However, it
                    // can match any number of non-'/' characters (even zero),
                    // so we must see what matches the remaining tokens up until
                    // the next SeqWildcard (or the end if no further SeqWildcards
                    // remain)

                    // If there are no more tokens left, make sure there is no '/'
                    // in the rest of the string
                    if tokens.peek().is_none() {
                        return !next.contains('/');
                    };

                    // For every remaining position in next until '/', check if
                    // the remaining tokens until SeqWildcard match.
                    'star: for (i, c) in next.char_indices() {
                        let mut tokens_peek = tokens.clone();
                        let mut next_peek = &next[i..];
                        let mut fail = false;
                        let mut finished = true;
                        'inner: while let Some(token_peek) = tokens_peek.peek() {
                            next_peek = match token_peek.try_matches_next(next_peek) {
                                Ok(Some(next_peek)) => next_peek,
                                Ok(None) => {
                                    fail = true;
                                    break 'inner;
                                }
                                Err(_) => {
                                    finished = false;
                                    break 'inner;
                                }
                            };
                            tokens_peek.next();
                        }

                        if !fail && (!finished || next_peek.is_empty()) {
                            // Either we correctly matched until the next SeqWildcard,
                            // or there are no tokens left and the entirety of the
                            // string is matched. In either case we continue
                            tokens = tokens_peek;
                            next = next_peek;
                            continue 'outer;
                        }

                        // Match failed, try from next position.

                        if c == '/' {
                            // Found '/', abort
                            break 'star;
                        }
                    }

                    // Exhausted available positions without finding a match.
                    return false;
                }
            }
        }
        next.is_empty()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GlobToken {
    Literal(GlobTokenLiteral),
    CharClass(GlobTokenCharClass),
    SeqWildcard,
    SingleWildcard,
}
impl GlobToken {
    fn try_matches_next<'a>(&self, name: &'a str) -> StdResult<Option<&'a str>, ()> {
        match self {
            GlobToken::Literal(l) => Ok(l.matches_next(name)),
            GlobToken::CharClass(cc) => Ok(cc.matches_next(name)),
            GlobToken::SingleWildcard => Ok(name.strip_prefix(|c| c != '/')),
            GlobToken::SeqWildcard => Err(()),
        }
    }
}

/// Internal workspace-only function employed by `goglob-proc-macro`.
///
/// The procedural macro will insert calls to this function in the end-user's project,
/// so it must be declared public.
pub const fn glob_from_tokens(tokens: &'static [GlobToken]) -> GlobPattern {
    GlobPattern {
        tokens: Cow::Borrowed(tokens),
    }
}

/// Internal workspace-only function used locally and in `goglob-proc-macro`.
pub fn scan_patterns(pattern: &str, tokens: &mut Vec<GlobToken>) -> Result<()> {
    if pattern.is_empty() {
        return Err(Error::empty_pattern());
    }

    let mut pattern_iter = pattern.char_indices().peekable();
    while pattern_iter.peek().is_some() {
        let mut stars = false;

        // Match star wildcards (e.g. '*ab?cd[e-z]*')
        //                             ^          ^
        while let Some((_, '*')) = pattern_iter.peek() {
            stars = true;
            pattern_iter.next();
        }
        if stars {
            tokens.push(GlobToken::SeqWildcard)
        }

        // Match literals (e.g. '*ab?cd[e-z]*')
        //                        ^^ ^^
        let mut literal_string = String::new();
        'literal: while let Some((i, c)) = pattern_iter.peek() {
            let (i, c) = (*i, *c);
            let c = match c {
                ']' =>
                // we are not in a character class (i.e. '[' was never passed)
                // therefore ']' is illegal and should be explicitly escaped
                {
                    return Err(Error::new(ErrorType::UnescapedChar(']'), i))
                }
                '[' | '?' | '*' =>
                // '[' opens a character class
                // '?' is a single-character wildcard
                // '*' is a multi-character wildcard
                // any of these signal an end to the current literal
                {
                    break 'literal
                }
                '\\' => {
                    pattern_iter.next();

                    // '\' escapes the next character, whichever it may be.
                    // If there is no "next character", then it's considered
                    // an illegal escape
                    let (_, escaped_char) = pattern_iter
                        .next()
                        .ok_or_else(|| Error::new(ErrorType::IllegalEscape, i))?;
                    escaped_char
                }
                c => {
                    pattern_iter.next();
                    c
                }
            };

            literal_string.push(c);
        }
        if !literal_string.is_empty() {
            tokens.push(GlobToken::Literal(GlobTokenLiteral::new(literal_string)))
        }

        // Match question-mark wildcards (e.g. '*ab?cd[e-z]*')
        //                                         ^
        while let Some((_, '?')) = pattern_iter.peek() {
            tokens.push(GlobToken::SingleWildcard);
            pattern_iter.next();
        }

        // Match character class (e.g. '*ab?cd[e-z]*')
        //                                    ^^^^^
        if let Some((i, '[')) = pattern_iter.peek() {
            let mut types: Vec<CharClassType> = Vec::new();
            let mut negated = false;
            let mut closed = false;
            let i = *i;
            let start_i = i;
            let mut closed_i = usize::MAX;

            pattern_iter.next();

            // Match negation in character class (e.g. '[^A-F]')
            //                                           ^
            if let Some((_, '^')) = pattern_iter.peek() {
                pattern_iter.next();
                negated = true;
            }

            let mut start_range = false;
            let mut in_range: Option<char> = None;
            'char_cls: while let Some((i, c)) = pattern_iter.next() {
                debug_assert!(!start_range || c == '-');
                let c = match c {
                    ']' => {
                        // Close the character range
                        closed = true;
                        closed_i = i;
                        break 'char_cls;
                    }
                    '^' =>
                    // The character class was already started or negated.
                    // Either way, a '^' here must be interpreted as a
                    // character as-is (at least according to go's impl).
                    {
                        '^'
                    }
                    '-' if !start_range =>
                    // Illegal uses of '-':
                    //
                    // * As the first character in the class (e.g. [-a][^-z])
                    //                                              ^    ^
                    // * After another '-' (e.g. [a--f])
                    //                              ^
                    // * Immediately after a character range (e.g. [a-f-z])
                    //                                                 ^
                    // If a literal '-' is desired, escape it with a '\' beforehand
                    // (e.g. [a-f\-z][\-a][^\-z])
                    //           ^^   ^^    ^^
                    {
                        return Err(Error::new(ErrorType::UnescapedChar('-'), i))
                    }
                    '-' => {
                        // Character range (e.g. [0-9abcdefA-F]
                        //                        ^^^      ^^^
                        start_range = false;
                        continue 'char_cls;
                    }
                    '\\' => {
                        // '\' escapes the next character, whichever it may be.
                        // If there is no "next character", then it's considered
                        // an illegal escape
                        let (_, escaped_char) = pattern_iter
                            .next()
                            .ok_or_else(|| Error::new(ErrorType::IllegalEscape, i))?;
                        escaped_char
                    }
                    c => c,
                };
                if let Some(start) = in_range {
                    let end = c;
                    let range = (start..=end)
                        .try_into()
                        .map_err(|_| Error::new(ErrorType::InvalidRangeValues(start, end), i))?;
                    types.push(range);
                    in_range = None
                } else if let Some((_, '-')) = pattern_iter.peek() {
                    in_range = Some(c);
                    start_range = true
                } else {
                    types.push(c.into())
                }
            }

            // A character class must be closed with a corresponding ']'.
            if !closed {
                return Err(Error::new(ErrorType::UnclosedCharClass, start_i));
            }

            // A character class must not be empty (e.g. []abc] or [^]abc])
            //                                            ^          ^
            // For the character class to include a ']' char it must be
            // explicitly escaped (e.g. [\]abc] or [^\]abc].
            //                           ^^          ^^
            if types.is_empty() {
                return Err(Error::new(ErrorType::UnescapedChar(']'), closed_i));
            }

            tokens.push(GlobToken::CharClass(GlobTokenCharClass::new(
                negated, types,
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
//noinspection DuplicatedCode
mod tests {
    mod aux {
        use crate::{GlobPattern, Result as GlobPatternResult};
        use std::fmt::{Display, Formatter};

        #[derive(Clone)]
        pub struct MatchTest {
            pattern: String,
            name: String,
            expect_match: Result<bool, ()>,
        }
        impl MatchTest {
            const fn _new(pattern: String, name: String, expect_match: Result<bool, ()>) -> Self {
                Self {
                    pattern,
                    name,
                    expect_match,
                }
            }

            pub fn display(&self) -> TestDisplay {
                let clone = self.clone();
                TestDisplay { test: clone }
            }

            pub fn test(&self) -> GlobPatternResult<bool> {
                GlobPattern::new(self.pattern.clone()).map(|p| p.matches(self.name.clone()))
            }

            pub fn succeed(self, result: GlobPatternResult<bool>) -> bool {
                result.map_err(|_| ()) == self.expect_match
            }
        }
        #[inline]
        pub fn make_test<S1: Into<String>, S2: Into<String>>(
            pattern: S1,
            name: S2,
            expect_match: Result<bool, ()>,
        ) -> MatchTest {
            MatchTest::_new(pattern.into(), name.into(), expect_match)
        }

        pub struct TestDisplay {
            test: MatchTest,
        }
        impl Display for TestDisplay {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "({}, {}) expected {:?}",
                    self.test.pattern, self.test.name, self.test.expect_match,
                )
            }
        }
    }

    use aux::*;

    #[test]
    fn glob_pattern_go_match_test() {
        let tests = [
            make_test("abc", "abc", Ok(true)),
            make_test("*", "abc", Ok(true)),
            make_test("*c", "abc", Ok(true)),
            make_test("a*", "a", Ok(true)),
            make_test("a*", "abc", Ok(true)),
            make_test("a*", "ab/c", Ok(false)),
            make_test("a*/b", "abc/b", Ok(true)),
            make_test("a*/b", "a/c/b", Ok(false)),
            make_test("a*b*c*d*e*/f", "axbxcxdxe/f", Ok(true)),
            make_test("a*b*c*d*e*/f", "axbxcxdxexxx/f", Ok(true)),
            make_test("a*b*c*d*e*/f", "axbxcxdxe/xxx/f", Ok(false)),
            make_test("a*b*c*d*e*/f", "axbxcxdxexxx/fff", Ok(false)),
            make_test("a*b?c*x", "abxbbxdbxebxczzx", Ok(true)),
            make_test("a*b?c*x", "abxbbxdbxebxczzy", Ok(false)),
            make_test("ab[c]", "abc", Ok(true)),
            make_test("ab[b-d]", "abc", Ok(true)),
            make_test("ab[e-g]", "abc", Ok(false)),
            make_test("ab[^c]", "abc", Ok(false)),
            make_test("ab[^b-d]", "abc", Ok(false)),
            make_test("ab[^e-g]", "abc", Ok(true)),
            make_test("a\\*b", "a*b", Ok(true)),
            make_test("a\\*b", "ab", Ok(false)),
            make_test("a?b", "a☺b", Ok(true)),
            make_test("a[^a]b", "a☺b", Ok(true)),
            make_test("a???b", "a☺b", Ok(false)),
            make_test("a[^a][^a][^a]b", "a☺b", Ok(false)),
            make_test("[a-ζ]*", "α", Ok(true)),
            make_test("*[a-ζ]", "A", Ok(false)),
            make_test("a?b", "a/b", Ok(false)),
            make_test("a*b", "a/b", Ok(false)),
            make_test("[\\]a]", "]", Ok(true)),
            make_test("[\\-]", "-", Ok(true)),
            make_test("[x\\-]", "x", Ok(true)),
            make_test("[x\\-]", "-", Ok(true)),
            make_test("[x\\-]", "z", Ok(false)),
            make_test("[\\-x]", "x", Ok(true)),
            make_test("[\\-x]", "-", Ok(true)),
            make_test("[\\-x]", "a", Ok(false)),
            make_test("[]a]", "]", Err(())),
            make_test("[-]", "-", Err(())),
            make_test("[x-]", "x", Err(())),
            make_test("[x-]", "-", Err(())),
            make_test("[x-]", "z", Err(())),
            make_test("[-x]", "x", Err(())),
            make_test("[-x]", "-", Err(())),
            make_test("[-x]", "a", Err(())),
            make_test("\\", "a", Err(())),
            make_test("[a-b-c]", "a", Err(())),
            make_test("[", "a", Err(())),
            make_test("[^", "a", Err(())),
            make_test("[^bc", "a", Err(())),
            make_test("a[", "a", Err(())),
            make_test("a[", "ab", Err(())),
            make_test("a[", "x", Err(())),
            make_test("a/b[", "x", Err(())),
            make_test("*x", "xxx", Ok(true)),
        ];

        for (i, test) in tests.into_iter().enumerate() {
            let display = test.display();
            let result = test.test();
            let result_display = format!("{:?}", result);
            assert!(
                test.succeed(result),
                "[Test {i}]: {display}, got {result_display}"
            )
        }
    }
}
