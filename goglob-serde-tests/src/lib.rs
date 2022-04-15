//! # Do not use this crate!
//!
//! See the `goglob` crate instead.
//!
//! (This crate facilitates testing `goglob`'s serde deserialization
//! functionality with `cargo test`. It offers no functionality to the
//! end user)

pub fn stub_sub(a: usize, b: usize) -> usize {
    a - b
}

#[cfg(test)]
//noinspection DuplicatedCode
mod tests {
    mod aux {
        use goglob::{GlobPattern, Result as GlobPatternResult};
        use serde::{Deserialize, Serialize};
        use std::fmt::{Display, Formatter};

        #[derive(Deserialize)]
        pub struct DeserializedPattern {
            pub pattern: GlobPattern,
        }

        #[derive(Serialize)]
        pub struct SerializedPattern {
            pub pattern: String,
        }

        #[derive(Clone)]
        pub struct MatchTest {
            pattern: String,
            name: String,
            expect_match: Option<bool>,
        }
        impl MatchTest {
            const fn _new(pattern: String, name: String, expect_match: Option<bool>) -> Self {
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

            pub fn test(&self) -> Option<bool> {
                let pattern1: GlobPatternResult<GlobPattern> =
                    GlobPattern::new(self.pattern.clone());
                let pattern2: Option<GlobPattern> = serde_json::from_str::<DeserializedPattern>(
                    &*serde_json::to_string(&SerializedPattern {
                        pattern: self.pattern.clone(),
                    })
                    .unwrap(),
                )
                .ok()
                .map(|p| p.pattern);

                if pattern1.is_ok() == pattern2.is_some() {
                    pattern1
                        .map(|p| p.matches(self.name.clone()))
                        .ok()
                        .filter(|r| *r == pattern2.unwrap().matches(self.name.clone()))
                } else {
                    None
                }
            }

            pub fn succeed(self, result: Option<bool>) -> bool {
                result == self.expect_match
            }
        }
        #[inline]
        pub fn make_test<S1: Into<String>, S2: Into<String>>(
            pattern: S1,
            name: S2,
            expect_match: Option<bool>,
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
    fn serde_go_match_test() {
        let tests = [
            make_test("abc", "abc", Some(true)),
            make_test("*", "abc", Some(true)),
            make_test("*c", "abc", Some(true)),
            make_test("a*", "a", Some(true)),
            make_test("a*", "abc", Some(true)),
            make_test("a*", "ab/c", Some(false)),
            make_test("a*/b", "abc/b", Some(true)),
            make_test("a*/b", "a/c/b", Some(false)),
            make_test("a*b*c*d*e*/f", "axbxcxdxe/f", Some(true)),
            make_test("a*b*c*d*e*/f", "axbxcxdxexxx/f", Some(true)),
            make_test("a*b*c*d*e*/f", "axbxcxdxe/xxx/f", Some(false)),
            make_test("a*b*c*d*e*/f", "axbxcxdxexxx/fff", Some(false)),
            make_test("a*b?c*x", "abxbbxdbxebxczzx", Some(true)),
            make_test("a*b?c*x", "abxbbxdbxebxczzy", Some(false)),
            make_test("ab[c]", "abc", Some(true)),
            make_test("ab[b-d]", "abc", Some(true)),
            make_test("ab[e-g]", "abc", Some(false)),
            make_test("ab[^c]", "abc", Some(false)),
            make_test("ab[^b-d]", "abc", Some(false)),
            make_test("ab[^e-g]", "abc", Some(true)),
            make_test("a\\*b", "a*b", Some(true)),
            make_test("a\\*b", "ab", Some(false)),
            make_test("a?b", "a☺b", Some(true)),
            make_test("a[^a]b", "a☺b", Some(true)),
            make_test("a???b", "a☺b", Some(false)),
            make_test("a[^a][^a][^a]b", "a☺b", Some(false)),
            make_test("[a-ζ]*", "α", Some(true)),
            make_test("*[a-ζ]", "A", Some(false)),
            make_test("a?b", "a/b", Some(false)),
            make_test("a*b", "a/b", Some(false)),
            make_test("[\\]a]", "]", Some(true)),
            make_test("[\\-]", "-", Some(true)),
            make_test("[x\\-]", "x", Some(true)),
            make_test("[x\\-]", "-", Some(true)),
            make_test("[x\\-]", "z", Some(false)),
            make_test("[\\-x]", "x", Some(true)),
            make_test("[\\-x]", "-", Some(true)),
            make_test("[\\-x]", "a", Some(false)),
            make_test("[]a]", "]", None),
            make_test("[-]", "-", None),
            make_test("[x-]", "x", None),
            make_test("[x-]", "-", None),
            make_test("[x-]", "z", None),
            make_test("[-x]", "x", None),
            make_test("[-x]", "-", None),
            make_test("[-x]", "a", None),
            make_test("\\", "a", None),
            make_test("[a-b-c]", "a", None),
            make_test("[", "a", None),
            make_test("[^", "a", None),
            make_test("[^bc", "a", None),
            make_test("a[", "a", None),
            make_test("a[", "ab", None),
            make_test("a[", "x", None),
            make_test("a/b[", "x", None),
            make_test("*x", "xxx", Some(true)),
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
