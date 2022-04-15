use crate::GlobTokenLiteral;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn glob_token_literal_into_stream(l: GlobTokenLiteral) -> TokenStream {
    let l = l.as_ref();
    quote!(
        ::goglob::internal::GlobToken::Literal(
            ::goglob::internal::literal::from_static(#l)
        )
    )
}
