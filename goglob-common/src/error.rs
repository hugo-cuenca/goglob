use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::result::Result as StdResult;

#[derive(Debug)]
pub struct Error {
    error_type: ErrorType,
    pos: usize,
}
impl Error {
    pub(crate) fn new(error_type: ErrorType, pos: usize) -> Self {
        Self { error_type, pos }
    }

    pub(crate) fn empty_pattern() -> Self {
        Self {
            error_type: ErrorType::EmptyPattern,
            pos: usize::MAX,
        }
    }

    pub fn error_type(&self) -> &ErrorType {
        &self.error_type
    }

    pub fn position(&self) -> usize {
        self.pos
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.error_type.fmt_with_pos(Some(self.pos), f)
    }
}
impl StdError for Error {}

#[derive(Copy, Clone)]
pub enum ErrorType {
    EmptyPattern,
    IllegalEscape,
    InvalidRangeValues(char, char),
    UnclosedCharClass,
    UnescapedChar(char),
}
impl ErrorType {
    pub fn type_desc(&self) -> &'static str {
        match self {
            ErrorType::EmptyPattern => "empty pattern",
            ErrorType::IllegalEscape => "illegal use of '\\': end of pattern",
            ErrorType::InvalidRangeValues(_, _) => "invalid character range",
            ErrorType::UnclosedCharClass => "character class opened with '[' isn't closed",
            ErrorType::UnescapedChar(_) => "special character not escaped with '\\'",
        }
    }

    pub fn full_desc(&self) -> String {
        format!("{}", self)
    }

    pub fn fmt_with_pos(&self, pos: Option<usize>, f: &mut Formatter<'_>) -> FmtResult {
        match (self, pos) {
            (ErrorType::IllegalEscape, Some(pos)) => {
                write!(f, "illegal use of '\\' at {pos}: end of pattern")
            }
            (ErrorType::InvalidRangeValues(start, end), Some(pos)) => {
                write!(f, "invalid charater range at {pos}: {start}-{end}")
            }
            (ErrorType::InvalidRangeValues(start, end), None) => {
                write!(f, "invalid charater range: {start}-{end}")
            }
            (ErrorType::UnclosedCharClass, Some(pos)) => {
                write!(f, "character class opened with '[' at {pos} isn't closed")
            }
            (ErrorType::UnescapedChar(unescaped), Some(pos)) => {
                write!(
                    f,
                    "special character {unescaped} at {pos} not escaped with '\\'"
                )
            }
            (ErrorType::UnescapedChar(unescaped), None) => {
                write!(f, "special character {unescaped} not escaped with '\\'")
            }
            (_, _) => f.write_str(self.type_desc()),
        }
    }
}
impl Debug for ErrorType {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.fmt_with_pos(None, f)
    }
}
impl Display for ErrorType {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.fmt_with_pos(None, f)
    }
}

pub type Result<T> = StdResult<T, Error>;
