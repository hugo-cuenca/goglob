use goglob_common::charcls::{CharClass as GlobTokenCharClass, CharClassType};
use proc_macro2::TokenStream;
use quote::quote;
use std::ops::RangeInclusive;

mod cct {
    use proc_macro2::{Span, TokenStream};
    use quote::quote;
    use std::ops::RangeInclusive;

    pub(super) fn cct_single_into_stream(c: char) -> TokenStream {
        let c = syn::LitChar::new(c, Span::call_site());
        quote!(
            ::goglob::internal::charcls::type_from_char(#c)
        )
    }

    pub(super) fn cct_range_into_stream(rg: RangeInclusive<char>) -> TokenStream {
        let start = syn::LitChar::new(*rg.start(), Span::call_site());
        let end = syn::LitChar::new(*rg.end(), Span::call_site());
        quote!(
            unsafe {
                ::goglob::internal::charcls::type_from_range_unchecked(
                    /* SAFETY guaranteed at compile-time */ #start..=#end
                )
            }
        )
    }
}

pub(crate) fn glob_token_char_class_into_stream(cc: GlobTokenCharClass) -> TokenStream {
    let negated = cc.is_negated();
    let mut inner_result = quote!();
    for char_class_type in cc {
        let new_append = match char_class_type {
            CharClassType::Single(sc) => cct::cct_single_into_stream(char::from(sc)),
            CharClassType::Range(srg) => {
                cct::cct_range_into_stream(RangeInclusive::<char>::from(srg))
            }
        };
        inner_result = quote![
            #inner_result
            #new_append,
        ]
    }
    let result = quote! {{
        const RESULTING_CCTS: &'static [::goglob::internal::charcls::CharClassType] = &[
            #inner_result
        ];
        ::goglob::internal::charcls::from_static(#negated, RESULTING_CCTS)
    }};
    quote!(
        ::goglob::internal::GlobToken::CharClass(
            #result
        )
    )
}
