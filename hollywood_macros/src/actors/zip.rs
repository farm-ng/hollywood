use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse2;
use syn::LitInt;
use syn::Result;

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

    let field_seq = (0..num_fields).map(|i| format_ident!("item{}", i));
    let field_seq2 = field_seq.clone();
    let field_seq3 = field_seq.clone();

    let type_seq = (0..num_fields).map(|i| format_ident!("Item{}", i));
    let type_seq2 = type_seq.clone();
    let type_seq3 = type_seq.clone();
    let type_with_bounds_seq = (0..num_fields).map(|i| {
        let ident = format_ident!("Item{}", i);
        quote! { #ident: Default + Clone + Debug + Sync + Send + 'static
        + std::marker::Sync+ std::marker::Send}
    });

    let expanded = quote! {
        #[derive(Default, Clone, Debug)]

        /// A tuple struct X fields.
        ///
        /// Used to send merged items from X inbound channels to one outbound channel.
        pub struct #tuple_struct<Key:Default+Clone+Debug, #( #type_seq ),*> {
            /// Key to associate message from different inbound channels with.
            pub key: Key,
            #(
                /// The value to be zipped.
                pub #field_seq: #type_seq3
            ),*
        }
        impl<
                Key: Default
                    + Debug
                    + Clone
                    + Display
                    + PartialEq
                    + Eq
                    + PartialOrd
                    + Ord
                    + Sync
                    + Send
                    + 'static,
                    #( #type_with_bounds_seq ),*
            > Display for #tuple_struct<Key, #( #type_seq2 ),*>
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    concat!( "key: {}", #( stringify!(, #field_seq2), ": {:?}" ),* ),
                    self.key,  #( self.#field_seq3), *)
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

    let params_seq = (0..num_fields).map(|i| format_ident!("Item{}", i));
    let params_seq2 = params_seq.clone();
    let params_seq3 = params_seq.clone();
    let params_seq4 = params_seq.clone();
    let params_seq5 = params_seq.clone();

    let params_with_bounds_seq: Vec<_> = (0..num_fields)
        .map(|i| {
            let ident = format_ident!("Item{}", i);
            quote! { #ident: Default + Clone + Debug + Sync + Send
            + 'static +std::marker::Sync+ std::marker::Send}
        })
        .collect();
    let params_with_bounds_seq2 = params_with_bounds_seq.clone();
    let params_with_bounds_seq3 = params_with_bounds_seq.clone();

    let expanded = quote! {
        /// ZipX outbound hub
        ///
        /// Contains one outbound channel of the merged inbound channels.
        pub struct #outbound_struct<Key:Default + Debug + Clone + Sync + Send
                                + 'static , #( #params_with_bounds_seq ),*> {
            /// Outbound channel of the merged inbound channels.
            pub zipped: OutboundChannel<#tuple_struct<Key, #( #params_seq2 ),*>>,
        }

        impl<Key: Default +  Debug+Clone+Sync + Send + 'static, #( #params_with_bounds_seq2 ),*>
            HasActivate for #outbound_struct<Key, #( #params_seq3 ),*>
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

        impl<Key:  Default + Debug+Clone+Sync + Send + 'static, #( #params_with_bounds_seq3 ),*>
            IsOutboundHub for #outbound_struct<Key, #( #params_seq4 ),*>
        {
            fn from_context_and_parent(context: &mut Hollywood, actor_name: &str) -> Self {
                Self {
                    zipped: OutboundChannel::<#tuple_struct<Key, #( #params_seq5 ),*>>::new(
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
        let item = format_ident!("Item{}", i); // Corrected this line
        let pair = quote! { ZipPair<#i, Key, #item> };
        quote! {
            /// Heap for the Xth inbound channel.
            pub #heap_item: std::collections::BinaryHeap<std::cmp::Reverse<#pair>>
        }
    });

    let item_seq = (0..num_fields).map(|i| format_ident!("Item{}", i));

    let expanded = quote! {

        /// State of the zip actor with X inbound channels.
        #[derive(Clone, Debug, Default)]
        pub struct #state_struct<Key: PartialEq + Eq + PartialOrd + Ord,
                                #( #item_seq: Default + Clone + Debug + Sync + Send + 'static ),*>
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

    let type_seq = (0..num_fields).map(|i| format_ident!("Item{}", i));
    let type_seq2 = type_seq.clone();
    let type_seq3 = type_seq.clone();
    let type_seq4 = type_seq.clone();
    let type_seq5 = type_seq.clone();
    let type_seq6 = type_seq.clone();
    let type_with_bounds_seq: Vec<_> = (0..num_fields)
        .map(|i| {
            let ident = format_ident!("Item{}", i);
            quote! { #ident: Default + Clone + Debug + Sync + Send
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
                   Key: Default
                       + Clone
                       + Debug
                       + PartialEq
                       + Eq
                       + PartialOrd
                       + Ord
                       + Debug
                       + Clone
                       + Sync
                       + Send
                       + 'static,
                       #( #type_with_bounds_seq ),*
                       > IsInboundMessageNew<ZipPair<#i, Key, #item>>
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
        #[derive(Clone,Debug)]
        pub enum #inbound_message_enum<
            Key: Ord + Clone + Debug + Sync + Send + 'static,
            #( #type_with_bounds_seq ),*
        > {
            #(
                /// Inbound message for the Xth inbound channel.
                #type_seq(ZipPair<#i_seq, Key, #type_seq>)
            ),*
        }

       #(#msg_new_impl_seq)*

       impl<
                Key: Default + Clone + Debug + PartialEq + Eq + PartialOrd + Ord
                + Sync + Send + 'static,
                #( #type_with_bounds_seq ),*
            > IsInboundMessage for  #inbound_message_enum<Key, #(#type_seq2),*>
        {
            type Prop = NullProp;
            type State = #state_struct<Key, #(#type_seq3),*>;
            type OutboundHub = #outbound_struct<Key, #(#type_seq4),*>;
            type RequestHub = NullRequest;

            fn inbound_channel(&self) -> String {
                match self {
                    #( #inbound_message_enum::#type_seq5(_) =>  stringify!(#type_seq6).to_owned(), )*
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

    let type_seq = (0..num_fields).map(|i| format_ident!("Item{}", i));
    let type_seq2 = type_seq.clone();
    let type_seq3 = type_seq.clone();
    let type_seq4 = type_seq.clone();
    let type_seq5 = type_seq.clone();
    let type_seq6 = type_seq.clone();
    let type_seq7 = type_seq.clone();
    let type_seq8 = type_seq.clone();
    let type_seq9 = type_seq.clone();
    let type_seq10 = type_seq.clone();
    let type_seq11 = type_seq.clone();
    let type_seq12 = type_seq.clone();

    let type_with_bounds_seq: Vec<_> = (0..num_fields)
        .map(|i| {
            let item_type = format_ident!("Item{}", i);
            quote! { #item_type: Default + Clone + Debug + Sync + Send
            + 'static}
        })
        .collect();

    let expanded = quote! {

        /// ZipX actor, which zips X inbound channels into one outbound channel.
        pub type #zip_struct<Key, #( #type_seq), *> = Actor<
                NullProp,
                #inbound_struct<Key, #( #type_seq2), *>,
                #state_struct<Key, #( #type_seq3), *>,
                #outbound_struct<Key, #( #type_seq4), *>,
                NullRequest,
            >;

        impl<
                Key: Default + Clone + Debug + PartialEq + Eq + PartialOrd + Ord
                     + Sync + Send + 'static,
                #( #type_with_bounds_seq ),*
            >
            HasFromPropState<
                NullProp,
                #inbound_struct<Key, #( #type_seq5), *>,
                #state_struct<Key, #( #type_seq6), *>,
                #outbound_struct<Key, #( #type_seq7), *>,
                #inbound_message_enum<Key, #( #type_seq8), *>,
                NullRequest,
                DefaultRunner<
                    NullProp,
                    #inbound_struct<Key, #( #type_seq9), *>,
                    #state_struct<Key, #( #type_seq10), *>,
                    #outbound_struct<Key, #( #type_seq11), *>,
                    NullRequest,
                >,
            > for #zip_struct<Key, #( #type_seq12), *>
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

    let type_seq = (0..num_fields).map(|i| format_ident!("Item{}", i));
    let type_seq4 = type_seq.clone();
    let type_seq5 = type_seq.clone();
    let type_seq6 = type_seq.clone();
    let type_seq7 = type_seq.clone();
    let type_seq8 = type_seq.clone();
    let type_seq9 = type_seq.clone();
    let type_seq10 = type_seq.clone();
    let type_seq12 = type_seq.clone();

    let item_seq = (0..num_fields).map(|i| format_ident!("item{}", i));
    let item_seq2 = item_seq.clone();
    let item_seq3 = item_seq.clone();
    let item_seq4 = item_seq.clone();
    let item_seq5 = item_seq.clone();

    let type_with_bounds_seq: Vec<_> = (0..num_fields)
        .map(|i| {
            let ident = format_ident!("Item{}", i);
            quote! { #ident: Default + Clone + Debug + Sync + Send
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
        #[derive(Clone,Debug)]
        pub struct #inbound_struct<
            Key: Default + Clone + Debug + PartialEq + Eq + PartialOrd + Ord
                 + Sync + Send + 'static,
            #( #type_with_bounds_seq ),*
        > {
            #(
                /// Inbound channel for the Xth inbound channel.
                pub #item_seq5: #channel
            ),*
        }

        impl<
                Key: Default + Clone + Debug + PartialEq + Eq + PartialOrd + Ord
                     + Sync + Send + 'static,
                #( #type_with_bounds_seq ),*
            >
            IsInboundHub<
                NullProp,
                #state_struct<Key, #( #type_seq4),*>,
                #outbound_struct<Key, #( #type_seq5),*>,
                NullRequest,
                #inbound_message_enum<Key, #( #type_seq6),*>,
            > for #inbound_struct<Key, #( #type_seq7),*>
        {
            fn from_builder(
                builder: &mut ActorBuilder<
                    NullProp,
                    #state_struct<Key, #( #type_seq8),*>,
                    #outbound_struct<Key, #( #type_seq9),*>,
                    NullRequest,
                    #inbound_message_enum<Key, #( #type_seq10),*>,
                >,
                actor_name: &str,
            ) -> Self {
                #(
                let #item_seq = InboundChannel::new(
                    builder.context,
                    actor_name,
                    &builder.sender,
                    stringify!(#type_seq12).to_owned(),
                );
                builder
                    .forward
                    .insert(#item_seq2.name.clone(), Box::new(#item_seq3.clone()));
                )*

                Self { #( #item_seq4 ),* }
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

    let type_seq = (0..num_fields).map(|i| format_ident!("Item{}", i));

    let type_with_bounds_seq: Vec<_> = (0..num_fields)
        .map(|i| {
            let ident = format_ident!("Item{}", i);
            quote! {
                #ident: Default + Clone + Debug + Sync + Send + 'static
            }
        })
        .collect();

    let front_seq = (0..num_fields).map(|i| format_ident!("front{}", i));

    let item_seq = (0..num_fields).map(|i| format_ident!("item{}", i));
    let item_seq2 = item_seq.clone();
    let item_seq3 = item_seq.clone();

    let item_heap = (0..num_fields).map(|i| format_ident!("item{}_heap", i));
    let item_heap2 = item_heap.clone();
    let item_heap3 = item_heap.clone();

    let key_seq = (0..num_fields).map(|i| format_ident!("key{}", i));
    let key_seq2 = key_seq.clone();
    let key_seq3 = key_seq.clone();
    let key_seq4 = key_seq.clone();
    let key_seq5 = key_seq.clone();

    let case: Vec<_> = (0..num_fields)
        .map(|i| {
            let item_type = format_ident!("Item{}", i);

            let item_heap = format_ident!("item{}_heap", i);
            let item_heap_seq = (0..num_fields).map(|i| format_ident!("item{}_heap", i));

            quote! {
                #inbound_message_enum::#item_type(msg) => {
                    state.#item_heap.push(Reverse(msg));
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

        impl<Key: Default + Clone + Debug + PartialEq + Eq + PartialOrd + Ord + Sync + Send,
            #( #type_with_bounds_seq ),*> HasOnMessage for #inbound_message_enum<Key, #(#type_seq), *>
        {
            fn on_message(
                self,
                _prop: &Self::Prop,
                state: &mut Self::State,
                outbound: &Self::OutboundHub,
                _request: &Self::RequestHub)
            {
                let check_and_send = |s: &mut Self::State| {
                    #(
                    let #front_seq = s.#item_heap.peek().unwrap();
                    let #key_seq2 = #front_seq.0.key.clone();
                    )*

                    let mut min = key0.clone();
                    #(
                    min = std::cmp::min(#key_seq3.clone(), min.clone());
                    )*

                    if #(#key_seq4 == min) && * {
                        #(let #item_seq = s.#item_heap2.pop().unwrap();)*

                        outbound.zipped.send(#tuple_struct {
                            key: min,
                            #(#item_seq2 : #item_seq3.0.value),*
                        });
                        return;
                    }
                    #(
                    if #key_seq5 == min {
                        s.#item_heap3.pop();
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
