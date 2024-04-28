use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, parse::ParseStream, parse2, LitInt, Result};
struct ZipInput {
    num_fields: usize,
}

impl Parse for ZipInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let num_fields: LitInt = input.parse()?;
        Ok(ZipInput {
            num_fields: num_fields.base10_parse()?,
        })
    }
}

pub(crate) fn tuple_n_impl(input: TokenStream) -> TokenStream {
    let ZipInput { num_fields } = match parse2(input) {
        Ok(input) => input,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let tuple_struct = format_ident!("Tuple{}", num_fields);

    // Collecting iterators into vectors to reuse them
    let field_seq: Vec<_> = (0..num_fields)
        .map(|i| format_ident!("item{}", i))
        .collect();
    let type_seq: Vec<_> = (0..num_fields)
        .map(|i| format_ident!("Item{}", i))
        .collect();
    let type_with_bounds_seq: Vec<_> = type_seq
        .iter()
        .map(|ident| {
            quote! {
                #ident: Default + Clone + std::fmt::Debug + Sync + Send + 'static
            }
        })
        .collect();

    let expanded = quote! {
        #[derive(Default, Clone, std::fmt::Debug)]
        /// A tuple struct with N fields.
        /// Used to send merged items from N inbound channels to one outbound channel.
        pub struct #tuple_struct<Key, #( #type_seq ),*>
        where
            Key: Default + Clone + std::fmt::Debug,
        {
            /// Key to associate messages from different inbound channels.
            pub key: Key,
            #(
                /// The value to be zipped.
                pub #field_seq: #type_seq
            ),*
        }

        impl<Key, #( #type_seq ),*> std::fmt::Display for #tuple_struct<Key, #( #type_seq ),*>
        where
            Key: Default + Clone + std::fmt::Debug + std::fmt::Display + PartialEq + Eq
                 + PartialOrd + Ord + Sync + Send + 'static,
            #( #type_with_bounds_seq ),*
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    concat!("key: {}", #( ", " , stringify!(#field_seq), ": {:?}" ),*),
                    self.key, #( &self.#field_seq ),*
                )
            }
        }
    };

    TokenStream::from(expanded)
}

pub(crate) fn zip_outbound_n_impl(input: TokenStream) -> TokenStream {
    let ZipInput { num_fields } = match parse2(input) {
        Ok(input) => input,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let outbound_struct = format_ident!("Zip{}Outbound", num_fields);
    let tuple_struct = format_ident!("Tuple{}", num_fields);

    // Generate parameters and collect into a vector for reuse
    let params_seq: Vec<_> = (0..num_fields)
        .map(|i| format_ident!("Item{}", i))
        .collect();
    let params_with_bounds_seq: Vec<_> = params_seq
        .iter()
        .map(|ident| {
            quote! {
                #ident: Default + Clone + std::fmt::Debug + Sync + Send + 'static
            }
        })
        .collect();

    let expanded = quote! {
        /// ZipN outbound hub
        ///
        /// Contains one outbound channel of the merged inbound channels.
        pub struct #outbound_struct<
            Key: Default + std::fmt::Debug + Clone + Sync + Send + 'static,
            #( #params_with_bounds_seq ),*
        > {
            /// Outbound channel of the merged inbound channels.
            pub zipped: OutboundChannel< #tuple_struct<Key, #( #params_seq ),*> >,
        }

        impl<
            Key: Default + std::fmt::Debug + Clone+Sync + Send + 'static,
            #( #params_with_bounds_seq ),*
        >
            HasActivate for #outbound_struct<Key, #( #params_seq ),*>
        {
            fn extract(&mut self) -> Self {
                Self {
                    zipped: self.zipped.extract(),
                }
            }

            fn activate(&mut self) {
                self.zipped.activate();
            }
        }

        impl<
            Key: Default + std::fmt::Debug + Clone + Sync + Send + 'static,
            #( #params_with_bounds_seq ),*
        >
            IsOutboundHub for #outbound_struct<Key, #( #params_seq ),*>
        {
            fn from_context_and_parent(context: &mut Hollywood, actor_name: &str) -> Self {
                Self {
                    zipped: OutboundChannel::<#tuple_struct<Key, #( #params_seq ),*>>::new(
                        context,
                        "zipped".to_owned(),
                        actor_name,
                    ),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

pub(crate) fn zip_state_n_impl(input: TokenStream) -> TokenStream {
    let ZipInput { num_fields } = match parse2(input) {
        Ok(input) => input,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let state_struct = format_ident!("Zip{}State", num_fields);

    let heap_item_seq = (0..num_fields).map(|i| {
        let heap_item = format_ident!("item{}_heap", i);
        let item = format_ident!("Item{}", i);
        let pair = quote! { ZipPair<#i, Key, #item> };
        quote! {
            /// Heap for the Nth inbound channel.
            pub #heap_item: std::collections::BinaryHeap<std::cmp::Reverse<#pair>>
        }
    });

    let item_seq = (0..num_fields).map(|i| format_ident!("Item{}", i));

    let expanded = quote! {
        /// State of the zip actor with N inbound channels.
        #[derive(Clone, std::fmt::Debug, Default)]
        pub struct #state_struct<
            Key: PartialEq + Eq + PartialOrd + Ord,
            #( #item_seq: Default + Clone + std::fmt::Debug + Sync + Send + 'static ),*
        >
        {
            #( #heap_item_seq ),*
        }
    };

    TokenStream::from(expanded)
}

pub(crate) fn zip_inbound_message_n_impl(input: TokenStream) -> TokenStream {
    let ZipInput { num_fields } = match parse2(input) {
        Ok(input) => input,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let inbound_message_enum = format_ident!("Zip{}IsInboundMessage", num_fields);
    let state_struct = format_ident!("Zip{}State", num_fields);
    let outbound_struct = format_ident!("Zip{}Outbound", num_fields);

    let type_seq: Vec<_> = (0..num_fields)
        .map(|i| format_ident!("Item{}", i))
        .collect();
    let type_with_bounds_seq: Vec<_> = (0..num_fields)
        .map(|i| {
            let ident = format_ident!("Item{}", i);
            quote! { #ident: Default + Clone + std::fmt::Debug + Sync + Send
            + 'static}
        })
        .collect();
    let i_seq: Vec<_> = (0..num_fields)
        .map(|i| {
            quote! { #i }
        })
        .collect();

    let msg_new_impl_seq = (0..num_fields).map(|i| {
        let item = format_ident!("Item{}", i);
        let type_seq = (0..num_fields).map(|i| format_ident!("Item{}", i));

        quote! {
            impl<
                Key: Default + Clone + std::fmt::Debug + PartialEq + Eq + PartialOrd
                    + Ord + Sync + Send + 'static,
                #( #type_with_bounds_seq ),*
            >
                IsInboundMessageNew<ZipPair<#i, Key, #item>>
                for #inbound_message_enum<Key, #( #type_seq ),*>
            {
                fn new(_inbound_name: String, msg: ZipPair<#i, Key, #item>) -> Self {
                    #inbound_message_enum::#item(msg)
                }
            }
        }
    });

    let expand = quote! {

        /// Inbound message for the zip actor.
        #[derive(Clone,std::fmt::Debug)]
        pub enum #inbound_message_enum<
            Key: Ord + Clone + std::fmt::Debug + Sync + Send + 'static,
            #( #type_with_bounds_seq ),*
        > {
            #(
                /// Inbound message for the Nth inbound channel.
                #type_seq(ZipPair<#i_seq, Key, #type_seq>)
            ),*
        }

       #(#msg_new_impl_seq)*

       impl<
            Key: Default + Clone + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord
                + Sync + Send + 'static,
            #( #type_with_bounds_seq ),*
        >
            IsInboundMessage for  #inbound_message_enum<Key, #(#type_seq),*>
        {
            type Prop = NullProp;
            type State = #state_struct<Key, #(#type_seq),*>;
            type OutboundHub = #outbound_struct<Key, #(#type_seq),*>;
            type OutRequestHub = NullOutRequests;

            fn inbound_channel(&self) -> String {
                match self {
                    #( #inbound_message_enum::#type_seq(_) => stringify!(#type_seq).to_owned(), )*
                }
            }
        }
    };

    TokenStream::from(expand)
}

pub(crate) fn zip_n_impl(input: TokenStream) -> TokenStream {
    let ZipInput { num_fields } = match parse2(input) {
        Ok(input) => input,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let zip_struct = format_ident!("Zip{}", num_fields);
    let state_struct = format_ident!("Zip{}State", num_fields);
    let inbound_struct = format_ident!("Zip{}Inbound", num_fields);
    let outbound_struct = format_ident!("Zip{}Outbound", num_fields);
    let inbound_message_enum = format_ident!("Zip{}IsInboundMessage", num_fields);

    let type_seq: Vec<_> = (0..num_fields)
        .map(|i| format_ident!("Item{}", i))
        .collect();

    let type_with_bounds_seq: Vec<_> = (0..num_fields)
        .map(|i| {
            let item_type = format_ident!("Item{}", i);
            quote! { #item_type: Default + Clone + std::fmt::Debug + Sync + Send
            + 'static}
        })
        .collect();

    let expanded = quote! {

        /// ZipN actor, which zips N inbound channels into one outbound channel.
        pub type #zip_struct<Key, #( #type_seq), *> =
            Actor<
                NullProp,
                #inbound_struct<Key, #( #type_seq), *>,
                NullInRequests,
                #state_struct<Key, #( #type_seq), *>,
                #outbound_struct<Key, #( #type_seq), *>,
                NullOutRequests,
            >;

        impl<
            Key: Default + Clone + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord
                + Sync + Send + 'static,
            #( #type_with_bounds_seq ),*
        >
            HasFromPropState<
                NullProp,
                #inbound_struct<Key, #( #type_seq ), *>,
                NullInRequests,
                #state_struct<Key, #( #type_seq ), *>,
                #outbound_struct<Key, #( #type_seq ), *>,
                #inbound_message_enum<Key, #( #type_seq ), *>,
                NullInRequestMessage,
                NullOutRequests,
                DefaultRunner<
                    NullProp,
                    #inbound_struct<Key, #( #type_seq ), *>,
                    NullInRequests,
                    #state_struct<Key, #( #type_seq ), *>,
                    #outbound_struct<Key, #( #type_seq ), *>,
                    NullOutRequests,
                >,
            > for #zip_struct<Key, #( #type_seq ), *>
        {
            fn name_hint(_prop: &NullProp) -> String {
                stringify!(#zip_struct).to_owned()
            }
        }

    };

    TokenStream::from(expanded)
}

pub(crate) fn zip_inbound_n_impl(input: TokenStream) -> TokenStream {
    let ZipInput { num_fields } = match parse2(input) {
        Ok(input) => input,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let inbound_struct = format_ident!("Zip{}Inbound", num_fields);
    let state_struct = format_ident!("Zip{}State", num_fields);
    let outbound_struct = format_ident!("Zip{}Outbound", num_fields);
    let inbound_message_enum = format_ident!("Zip{}IsInboundMessage", num_fields);

    let type_seq: Vec<_> = (0..num_fields)
        .map(|i| format_ident!("Item{}", i))
        .collect();

    let item_seq: Vec<_> = (0..num_fields)
        .map(|i| format_ident!("item{}", i))
        .collect();

    let type_with_bounds_seq: Vec<_> = (0..num_fields)
        .map(|i| {
            let ident = format_ident!("Item{}", i);
            quote! { #ident: Default + Clone + std::fmt::Debug + Sync + Send
            + 'static}
        })
        .collect();

    let channel: Vec<_> = (0..num_fields)
        .map(|i| {
            let item_type = format_ident!("Item{}", i);
            let type_seq = (0..num_fields).map(|i| format_ident!("Item{}", i));

            quote! {
                InboundChannel<ZipPair<#i, Key, #item_type>,
                              #inbound_message_enum<Key, #( #type_seq),*>>
            }
        })
        .collect();

    let expanded = quote! {

        /// Inbound hub for the zip actor.
        #[derive(Clone, std::fmt::Debug)]
        pub struct #inbound_struct<
            Key: Default + Clone + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord
                 + Sync + Send + 'static,
            #( #type_with_bounds_seq ),*
        > {
            #(
                /// Inbound channel for the Nth inbound channel.
                pub #item_seq: #channel
            ),*
        }

        impl<
                Key: Default + Clone + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord
                     + Sync + Send + 'static,
                #( #type_with_bounds_seq ),*
            >
            IsInboundHub<
                NullProp,
                #state_struct<Key, #( #type_seq ),*>,
                #outbound_struct<Key, #( #type_seq ),*>,
                NullOutRequests,
                #inbound_message_enum<Key, #( #type_seq ),*>,
                NullInRequestMessage,
            > for #inbound_struct<Key, #( #type_seq ),*>
        {
            fn from_builder(
                builder: &mut ActorBuilder<
                    NullProp,
                    #state_struct<Key, #( #type_seq ),*>,
                    #outbound_struct<Key, #( #type_seq ),*>,
                    NullOutRequests,
                    #inbound_message_enum<Key, #( #type_seq ),*>,
                    NullInRequestMessage,
                >,
                actor_name: &str,
            ) -> Self {
                #(
                let #item_seq = InboundChannel::new(
                    builder.context,
                    actor_name,
                    &builder.sender,
                    stringify!(#type_seq).to_owned(),
                );
                builder
                    .forward
                    .insert(#item_seq.name.clone(), Box::new(#item_seq.clone()));
                )*

                Self { #( #item_seq ),* }
            }
        }
    };

    TokenStream::from(expanded)
}

pub(crate) fn zip_onmessage_n_impl(input: TokenStream) -> TokenStream {
    let ZipInput { num_fields } = match parse2(input) {
        Ok(input) => input,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    let tuple_struct = format_ident!("Tuple{}", num_fields);
    let inbound_message_enum = format_ident!("Zip{}IsInboundMessage", num_fields);

    let type_seq: Vec<_> = (0..num_fields)
        .map(|i| format_ident!("Item{}", i))
        .collect();

    let type_with_bounds_seq: Vec<_> = (0..num_fields)
        .map(|i| {
            let ident = format_ident!("Item{}", i);
            quote! {
                #ident: Default + Clone + std::fmt::Debug + Sync + Send + 'static
            }
        })
        .collect();

    let front_seq = (0..num_fields).map(|i| format_ident!("front{}", i));

    let item_seq: Vec<_> = (0..num_fields)
        .map(|i| format_ident!("item{}", i))
        .collect();

    let item_heap: Vec<_> = (0..num_fields)
        .map(|i| format_ident!("item{}_heap", i))
        .collect();

    let key_seq: Vec<_> = (0..num_fields).map(|i| format_ident!("key{}", i)).collect();

    let case: Vec<_> = (0..num_fields)
        .map(|i| {
            let item_type = format_ident!("Item{}", i);

            let item_heap = format_ident!("item{}_heap", i);
            let item_heap_seq = (0..num_fields).map(|i| format_ident!("item{}_heap", i));

            quote! {
                #inbound_message_enum::#item_type(msg) => {
                    state.#item_heap.push(std::cmp::Reverse(msg));
                    loop {
                        if #( state.#item_heap_seq.len() == 0 )||*
                        {
                            break;
                        }
                        check_and_send(state);
                    }
                }
            }
        })
        .collect();

    let expanded = quote! {

        impl<
            Key: Default + Clone + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord + Sync
                + Send,
            #( #type_with_bounds_seq ),*
        >
            HasOnMessage for #inbound_message_enum<Key, #(#type_seq), *>
        {
            fn on_message(
                self,
                _prop: &Self::Prop,
                state: &mut Self::State,
                outbound: &Self::OutboundHub,
                _request: &Self::OutRequestHub)
            {
                let check_and_send = |s: &mut Self::State| {
                    #(
                        let #front_seq = s.#item_heap.peek().unwrap();
                        let #key_seq = #front_seq.0.key.clone();
                    )*

                    let mut min = key0.clone();
                    #(
                        min = std::cmp::min(#key_seq.clone(), min.clone());
                    )*

                    if #(#key_seq == min) && * {
                        #(
                            let #item_seq = s.#item_heap.pop().unwrap();
                        )*

                        outbound.zipped.send(#tuple_struct {
                            key: min,
                            #(#item_seq : #item_seq.0.value),*
                        });
                        return;
                    }
                    #(
                    if #key_seq == min {
                        s.#item_heap.pop();
                    }
                    )*
                };

                match self {
                    #( #case )*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
