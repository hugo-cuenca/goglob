mod charcls;
mod literal;

use goglob_common::GlobToken;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn glob_tokens_into_stream(glob_tokens: Vec<GlobToken>) -> TokenStream {
    let mut inner_result = quote!();
    for glob_token in glob_tokens {
        let new_append = match glob_token {
            GlobToken::Literal(l) => literal::glob_token_literal_into_stream(l),
            GlobToken::CharClass(cc) => charcls::glob_token_char_class_into_stream(cc),
            GlobToken::SeqWildcard => glob_token_seq_wildcard_into_stream(),
            GlobToken::SingleWildcard => glob_token_single_wildcard_into_stream(),
        };
        inner_result = quote![
            #inner_result
            #new_append,
        ]
    }

    let result = quote! {{
        const RESULTING_TOKENS: &'static [::goglob::internal::GlobToken] = &[
            #inner_result
        ];
        ::goglob::internal::glob_from_tokens(RESULTING_TOKENS)
    }};
    result
}

pub(crate) fn glob_token_seq_wildcard_into_stream() -> TokenStream {
    quote!(::goglob::internal::GlobToken::SeqWildcard)
}

pub(crate) fn glob_token_single_wildcard_into_stream() -> TokenStream {
    quote!(::goglob::internal::GlobToken::SingleWildcard)
}
