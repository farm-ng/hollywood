#![deny(missing_docs)]

//! Convenience macros for the Hollywood actor framework.

mod actors;
mod core;

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// Documented in the root-level hollywood crate.
#[proc_macro_attribute]
pub fn actor_outputs(attr: TokenStream, item: TokenStream) -> TokenStream {
    core::actor_outputs_impl(
        proc_macro2::TokenStream::from(attr),
        proc_macro2::TokenStream::from(item),
    )
    .into()
}

/// Documented in the root-level hollywood crate.
#[proc_macro_attribute]
pub fn actor_requests(attr: TokenStream, item: TokenStream) -> TokenStream {
    core::actor_requests_impl(
        proc_macro2::TokenStream::from(attr),
        proc_macro2::TokenStream::from(item),
    )
    .into()
}

/// Documented in the root-level hollywood crate.
#[proc_macro_attribute]
pub fn actor_inputs(args: TokenStream, inbound: TokenStream) -> TokenStream {
    core::actor_inputs_impl(
        proc_macro2::TokenStream::from(args),
        proc_macro2::TokenStream::from(inbound),
    )
    .into()
}

/// Documented in the root-level hollywood crate.
#[proc_macro_attribute]
pub fn actor(attr: TokenStream, item: TokenStream) -> TokenStream {
    core::actor_impl(
        proc_macro2::TokenStream::from(attr),
        proc_macro2::TokenStream::from(item),
    )
    .into()
}

/// Documented in the root-level hollywood crate.
#[proc_macro]
pub fn zip_n(input: TokenStream) -> TokenStream {
    let parsed = proc_macro2::TokenStream::from(input.clone());
    let parsed2 = parsed.clone();
    let parsed3 = parsed.clone();
    let parsed4 = parsed.clone();
    let parsed5 = parsed.clone();
    let parsed6 = parsed.clone();
    let parsed7 = parsed.clone();

    let output_tuple_n = actors::zip::tuple_n_impl(parsed);
    let output_zip_outbound_n = actors::zip::zip_outbound_n_impl(parsed2);
    let output_zip_state_n = actors::zip::zip_state_n_impl(parsed3);
    let output_inbound_message_n = actors::zip::zip_inbound_message_n_impl(parsed4);
    let output_zip_n = actors::zip::zip_n_impl(parsed5);
    let output_zip_inbound_n = actors::zip::zip_inbound_n_impl(parsed6);
    let output_zip_onmessage_n_new = actors::zip::zip_onmessage_n_impl(parsed7);

    let combined_output = quote! {
        #output_tuple_n
        #output_zip_outbound_n
        #output_zip_state_n
        #output_inbound_message_n
        #output_zip_n
        #output_zip_inbound_n
        #output_zip_onmessage_n_new
    };

    TokenStream::from(combined_output)
}
