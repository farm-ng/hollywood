#![deny(missing_docs)]

//! Convenience macros for the Hollywood actor framework.

mod actors;
mod core;

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// This macro generates the boilerplate for the outbound hub struct it is applied to.
///
/// Macro template:
///
/// ``` text
/// #[actor_outputs]
/// pub struct OUTBOUND {
///     pub CHANNEL0: OutboundChannel<TYPE0>,
///     pub CHANNEL1: OutboundChannel<TYPE1>,
///     ...
/// }
/// ```
///
/// Here, OUTBOUND is the user-specified name of the struct. The struct shall be defined right
/// after the macro invocation. (Indeed, these types of macros are called "attribute macros".
/// They are applied to the item directly following them, in this case a struct.) The outbound
/// struct consists of a zero, one or more outbound channels. Each outbound channel has a
/// user-specified name CHANNEL* and a user specified type TYPE*.
///
/// Effect: The macro generates the [IsOutboundHub](crate::IsOutboundHub) and
/// [HasActivate](crate::HasActivate) implementations for the provided struct OUTBOUND.
///
/// This is the first of three macros to define an actor. The other two are [macro@actor_inputs]
/// and [macro@actor].
#[proc_macro_attribute]
pub fn actor_outputs(attr: TokenStream, item: TokenStream) -> TokenStream {
    core::actor_outputs_impl(
        proc_macro2::TokenStream::from(attr),
        proc_macro2::TokenStream::from(item),
    )
    .into()
}

/// This macro generates the boilerplate for the request hub struct it is applied to.
///
/// Macro template:
///
/// ``` text
/// #[actor_out_requests]
/// pub struct REQUEST {
///     pub CHANNEL0: OutRequestChannel<REQ_TYPE0, REPL_TYPE0, M0>,
///     pub CHANNEL1: OutRequestChannel<REQ_TYPE1, REPL_TYPE2, M1>,
///     ...
/// }
/// ```
///
/// Here, REQUEST is the user-specified name of the struct. The struct shall be defined right
/// after the macro invocation. The request struct consists of one or more request channels.
/// Each request channel has name CHANNEL*, a request type REQ_TYPE*, a reply type REPL_TYPE*,
/// and a message type M*.
///
/// Effect: The macro generates the [IsRequestHub](crate::IsRequestHub) and
/// [HasActivate](crate::HasActivate) implementations for the provided struct REQUEST.
#[proc_macro_attribute]
pub fn actor_out_requests(attr: TokenStream, item: TokenStream) -> TokenStream {
    core::actor_requests_impl(
        proc_macro2::TokenStream::from(attr),
        proc_macro2::TokenStream::from(item),
    )
    .into()
}

/// This macro generates the boilerplate for the inbound hub of an actor.
///
/// Macro template:
///
/// ``` text
/// #[derive(Clone, Debug)]
/// #[actor_inputs(
///     INBOUND,
///     {
///         PROP,
///         STATE,
///         OUTBOUND,
///         OUT_REQUESTS,
///         IN_REQUEST_MESSAGE,
///     })]
/// pub enum INBOUND_MESSAGE {
///   VARIANT0(TYPE0),
///   VARIANT1(TYPE1),
///   ...
/// }
/// ```
///
/// INBOUND_MESSAGE is the user-specified name of an enum which shall be defined right below the
/// macro invocation. The enum shall consist of a zero, one or more message variants. Each
/// variant has a user-specified name VARIANT* and type TYPE*.
///
/// Prerequisite:
///   - The OUTBOUND struct is defined and implements [IsOutboundHub](crate::IsOutboundHub)
///     and [HasActivate](crate::HasActivate), typically using the [macro@actor_outputs] macro.
///   - The OUT_REQUESTS struct is defined and implements [IsRequestHub](crate::IsRequestHub) and
///     [HasActivate](crate::HasActivate), e.g. using the [actor_out_requests] macro.
///   - The IN_REQUEST_MESSAGE struct is defined and implements
///     [IsInRequestMessage](crate::IsInRequestMessage).
///   - The PROP and STATE structs are defined.
///
/// Effects:
///   - The macro defines the struct INBOUND that contains an inbound channel field for each
///     variant of the INBOUND_MESSAGE enum, and implements the
///     [IsInboundHub](crate::IsInboundHub) trait for it.
///   - Implements the [IsInboundMessage](crate::IsInboundMessage) trait for INBOUND_MESSAGE.
#[proc_macro_attribute]
pub fn actor_inputs(args: TokenStream, inbound: TokenStream) -> TokenStream {
    core::actor_inputs_impl(
        proc_macro2::TokenStream::from(args),
        proc_macro2::TokenStream::from(inbound),
    )
    .into()
}

/// This macro generates the boilerplate to define an new actor type.
///
/// Macro template:
///
/// ``` text
/// #[actor(INBOUND_MESSAGE, IN_REQUEST_MESSAGE)]
/// type ACTOR = Actor<PROP, INBOUND, IN_REQUESTS, STATE, OUTBOUND, OUT_REQUEST>;
/// ```
///
/// Here, ACTOR is the user-specified name of the actor type. The actor type shall be defined
/// right after the macro invocation as an alias of [Actor](crate::Actor).
///
/// Prerequisites:
///   - The OUTBOUND struct is defined and implements [IsOutboundHub](crate::IsOutboundHub) and
///     [HasActivate](crate::HasActivate), e.g. using the [actor_outputs] macro.
///   - The OUT_REQUEST struct is defined and implements [IsOutRequestHub](crate::IsOutRequestHub)
///     and [HasActivate](crate::HasActivate), e.g. using the [actor_out_requests] macro.
///   - The INBOUND_MESSAGE enum is defined and implements
///     [IsInboundMessage](crate::IsInboundMessage), as well as the INBOUND struct is defined
///     and implements the [IsInboundHub](crate::IsInboundHub) trait, e.g through the
///     [actor_inputs] macro.
///   - The PROP and STATE structs are defined.
///
/// Effect:
///   - This macro implements the [HasFromPropState](crate::HasFromPropState) trait for the ACTOR
///     type.
///
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
