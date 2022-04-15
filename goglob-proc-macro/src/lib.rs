//! # Do not use this library directly!
//!
//! See the `goglob` crate instead.
//!
//! A substantial portion of the token parsing done in this crate is
//! based on the [`cstr!()` procedural macro](https://crates.io/crates/cstr)
//! (available under the MIT License).

// Note from cstr:
// While this isn't necessary when using Cargo >= 1.42, omitting it actually requires path-less
// `--extern proc_macro` to be passed to `rustc` when building this crate. Some tools may not do
// this correctly. So it's added as a precaution.
extern crate proc_macro;

mod parse;
mod stream;

use goglob_common::{
    error::Error as GlobTokenError, literal::Literal as GlobTokenLiteral, scan_patterns, GlobToken,
};
use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

pub(crate) mod internal {
    pub use goglob_common::*;
}

pub(crate) enum Error {
    GlobTokenError(Span, GlobTokenError),
    ParseError(parse::ParseError),
}
impl From<parse::ParseError> for Error {
    fn from(pe: parse::ParseError) -> Self {
        Self::ParseError(pe)
    }
}

/// Compile the given `pattern` into tokens at code-compile time, emitting a
/// `GlobPattern` on success or a compile-error if `pattern` is syntactically
/// invalid.
///
/// This is useful in contexts when the pattern is a known constant and can thus
/// be declared as such:
///
/// ```no_compile
/// const MY_PATTERN: GlobPattern = glob!("a*b*c*d*e*/f");
/// ```
///
/// That way, there is no runtime penalty when compiling the pattern for the first
/// time as it will be pre-compiled into the resulting binary.
///
/// # Further reading
///
/// See the `goglob` crate's documentation for the appropriate syntax, as well
/// as [goglob::error::Error] for possible syntax errors.
#[proc_macro]
pub fn glob(lit: RawTokenStream) -> RawTokenStream {
    let mut glob_tokens = Vec::new();
    let result_tokens = if let Err(e) = glob_tokens_from(lit.into(), &mut glob_tokens) {
        match e {
            Error::GlobTokenError(span, gte) => {
                let gte = format!("pattern malformed: {}", gte);
                quote_spanned!(span => compile_error!(#gte))
            }
            Error::ParseError(parse::ParseError(span, msg)) => quote_spanned!(
                span => compile_error!(#msg)
            ),
        }
    } else {
        stream::glob_tokens_into_stream(glob_tokens)
    };
    result_tokens.into()
}

fn glob_tokens_from(lit: TokenStream, glob_tokens: &mut Vec<GlobToken>) -> Result<(), Error> {
    let (pattern, span) = parse::parse_input(lit)?;
    scan_patterns(&*pattern, glob_tokens).map_err(|gte| Error::GlobTokenError(span, gte))
}
