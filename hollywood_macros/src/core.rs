use convert_case::Case;
use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse2;
use syn::Error;
use syn::Fields;
use syn::Generics;
use syn::Ident;
use syn::Item;
use syn::ItemEnum;
use syn::ItemStruct;
use syn::Path;
use syn::PathArguments;
use syn::Result;
use syn::Token;
use syn::Type;
use syn::TypePath;

pub(crate) fn actor_outputs_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = match parse2::<ItemStruct>(item) {
        Ok(ast) => ast,
        Err(err) => return err.to_compile_error(),
    };
    let struct_name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &ast.fields {
        Fields::Named(fields_named) => &fields_named.named,
        _ => panic!("`generate_outputs_trait` can only be used with structs with named fields"),
    };

    let output_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        if let Some(inner_ty) = is_output_type(&field.ty) {
            quote! {
                #field_name: OutboundChannel::<#inner_ty>::new(
                    context,
                    stringify!(#field_name).to_owned(),
                    actor_name,
                )
            }
        } else {
            panic!("field type must be OutboundChannel<T>.");
        }
    });

    let output_extract = fields.iter().map(|field| {
        let field_name = &field.ident;

        quote! {
            #field_name: self.#field_name.extract()
        }
    });

    let output_act = fields.iter().map(|field| {
        let field_name = &field.ident;

        quote! {
            self.#field_name.activate();
        }
    });

    let gen = quote! {
        impl #impl_generics IsOutboundHub for #struct_name #ty_generics #where_clause {
            fn from_context_and_parent(context: &mut Hollywood, actor_name: &str) -> Self {
                Self {
                    #(#output_assignments),*
                }
            }
        }

        impl #impl_generics HasActivate for #struct_name #ty_generics #where_clause {
            fn extract(&mut self) -> Self {
                Self {
                    #(#output_extract),*
                }
            }

            fn activate(&mut self) {
                #(#output_act)*
            }
        }
        #ast
    };

    gen.into()
}

// This function checks if the field's type is OutboundChannel<T> and return T if it is
fn is_output_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if segments.len() == 1 && segments[0].ident == "OutboundChannel" {
            if let PathArguments::AngleBracketed(args) = &segments[0].arguments {
                if args.args.len() == 1 {
                    if let syn::GenericArgument::Type(inner_ty) = args.args.first().unwrap() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn actor_out_requests_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = match parse2::<ItemStruct>(item) {
        Ok(ast) => ast,
        Err(err) => return err.to_compile_error(),
    };
    let struct_name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &ast.fields {
        Fields::Named(fields_named) => &fields_named.named,
        _ => panic!("`generate_outputs_trait` can only be used with structs with named fields"),
    };

    let request_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: OutRequestChannel::new(
                stringify!(#field_name).to_owned(),
                actor_name,
                sender,
            )
        }
    });

    let request_extract = fields.iter().map(|field| {
        let field_name = &field.ident;

        quote! {
            #field_name: self.#field_name.extract()
        }
    });

    let output_act = fields.iter().map(|field| {
        let field_name = &field.ident;

        quote! {
            self.#field_name.activate();
        }
    });

    let field0 = fields
        .first()
        .expect("Request struct must have at least one field");
    let m_type = is_out_request_type(&field0.ty).unwrap()[2];

    let gen = quote! {
        impl #impl_generics IsOutRequestHub<#m_type> for #struct_name #ty_generics #where_clause {
            fn from_parent_and_sender(
                actor_name: &str, sender: &tokio::sync::mpsc::UnboundedSender<#m_type>
            ) -> Self {
                Self {
                    #(#request_assignments),*
                }
            }
        }

        impl #impl_generics HasActivate for #struct_name #ty_generics #where_clause {
            fn extract(&mut self) -> Self {
                Self {
                    #(#request_extract),*
                }
            }

            fn activate(&mut self) {
                #(#output_act)*
            }
        }

        #ast
    };

    gen.into()
}

// This function checks if the field's type is OutRequestChannel<Request, Reply, M>
fn is_out_request_type(ty: &Type) -> Option<[&Type; 3]> {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if segments.len() == 1 && segments[0].ident == "OutRequestChannel" {
            if let PathArguments::AngleBracketed(args) = &segments[0].arguments {
                if args.args.len() == 3 {
                    let mut pop_iter = args.args.iter();
                    if let syn::GenericArgument::Type(request_ty) = pop_iter.nth(0).unwrap() {
                        if let syn::GenericArgument::Type(reply_ty) = pop_iter.nth(0).unwrap() {
                            if let syn::GenericArgument::Type(m_ty) = pop_iter.nth(0).unwrap() {
                                return Some([request_ty, reply_ty, m_ty]);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn actor_inputs_impl(args: TokenStream, inbound: TokenStream) -> TokenStream {
    let ActorInbound {
        struct_name,
        prop_type,
        state_type,
        output_type,
        request_type,
        request_message_type,
    } = match parse2::<ActorInbound>(args) {
        Ok(args) => args,
        Err(err) => return err.to_compile_error(),
    };
    let ast = match parse2::<ItemEnum>(inbound) {
        Ok(ast) => ast,
        Err(err) => return err.to_compile_error(),
    };

    let name = &ast.ident;
    let generics = &ast.generics;
    let fields = &ast.variants;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let inbound = fields.iter().map(|variant| {
        let variant_name = &variant.ident;
        let snake_case_variant_name_str = variant_name.to_string().to_case(Case::Snake);
        let snake_case_variant_name = Ident::new(&snake_case_variant_name_str, variant_name.span());
        let field_type = if let Fields::Unnamed(fields_unnamed) = &variant.fields {
            &fields_unnamed.unnamed[0].ty
        } else {
            panic!("Enum variants must be tuples");
        };

        let msg = format!(
            "`{}` channel field - autogenerated by the [actor_inputs] macro.",
            variant_name
        );
        quote! {
            #[doc = #msg]
            pub #snake_case_variant_name: InboundChannel<#field_type, #name #ty_generics>
        }
    });

    let match_arm = fields.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #name::#variant_name(_) => {
                stringify!(#variant_name).to_string()
            }
        }
    });

    let from_builder_inbounds = fields.iter().map(|variant| {
        let variant_name = &variant.ident;
        let snake_case_variant_name_str = variant_name.to_string().to_case(Case::Snake);
        let snake_case_variant_name = Ident::new(&snake_case_variant_name_str, variant_name.span());

        assert!(
            generics.params.len() <= 1,
            "Only zero or one generic parameter is supported, got {}",
            generics.params.len()
        );

        quote! {
            let #snake_case_variant_name = InboundChannel::new(
                &mut builder.context,
                actor_name.clone(),
                &builder.sender,
                stringify!(#variant_name).to_owned(),
            );
            builder.forward.insert(
                #snake_case_variant_name.name.clone(),
                Box::new(#snake_case_variant_name.clone())
            );
        }
    });

    let from_builder_init = fields.iter().map(|variant| {
        let variant_name = &variant.ident;
        let snake_case_variant_name_str = variant_name.to_string().to_case(Case::Snake);
        let snake_case_variant_name = Ident::new(&snake_case_variant_name_str, variant_name.span());

        quote! {
            #snake_case_variant_name,
        }
    });

    let gen = quote! {
        #ast

        /// Auto-generated inbound hub for actor.
        pub struct #struct_name #impl_generics #where_clause {
            #(#inbound),*
        }

        impl #impl_generics IsInboundMessage for #name #ty_generics #where_clause {
            type Prop = #prop_type;
            type State = #state_type;
            type OutboundHub = #output_type;
            type OutRequestHub = #request_type;

            fn inbound_channel(&self) -> String {
                match self {
                   #(#match_arm),*
                }
            }
        }

        impl #impl_generics IsInboundHub<
            #prop_type,
            #state_type,
            #output_type,
            #request_type,
            #name #ty_generics,
            #request_message_type> for #struct_name #ty_generics #where_clause
        {
            fn from_builder(
                builder: &mut ActorBuilder<
                    #prop_type,
                    #state_type,
                    #output_type,
                    #request_type,
                    #name #ty_generics,
                    #request_message_type
                >,
                actor_name: &str) -> Self
            {
                #(#from_builder_inbounds)*

                #struct_name {
                    #(#from_builder_init)*
                }
            }
        }
    };

    gen.into()
}

struct ActorInbound {
    struct_name: Ident,
    prop_type: Type,
    state_type: Type,
    output_type: Type,
    request_type: Type,
    request_message_type: Type,
}

impl Parse for ActorInbound {
    fn parse(inbound: ParseStream) -> Result<Self> {
        let struct_name: Ident = inbound.parse()?;
        let _: Generics = inbound.parse()?;
        let _: Token![,] = inbound.parse()?;
        let content;
        syn::braced!(content in inbound);
        let prop_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;
        let state_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;
        let output_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;
        let request_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;
        let request_message_type: Type = content.parse()?;
        Ok(ActorInbound {
            struct_name,
            prop_type,
            state_type,
            output_type,
            request_type,
            request_message_type,
        })
    }
}

pub fn actor_in_requests_impl(args: TokenStream, inbound: TokenStream) -> TokenStream {
    let ActorInRequests {
        struct_name,
        prop_type,
        state_type,
        output_type,
        request_type,
        message_type,
    } = match parse2::<ActorInRequests>(args) {
        Ok(args) => args,
        Err(err) => return err.to_compile_error(),
    };
    let ast = match parse2::<ItemEnum>(inbound) {
        Ok(ast) => ast,
        Err(err) => return err.to_compile_error(),
    };

    let name = &ast.ident;
    let generics = &ast.generics;
    let fields = &ast.variants;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let in_requests = fields.iter().map(|variant| {
        let variant_name = &variant.ident;
        let snake_case_variant_name_str = variant_name.to_string().to_case(Case::Snake);
        let snake_case_variant_name = Ident::new(&snake_case_variant_name_str, variant_name.span());
        let field_type = if let Fields::Unnamed(fields_unnamed) = &variant.fields {
            &fields_unnamed.unnamed[0].ty
        } else {
            panic!("Enum variants must be tuples");
        };

        let msg = format!(
            "`{}` channel field - autogenerated by the [actor_inputs] macro.",
            variant_name
        );
        quote! {
            #[doc = #msg]
            pub #snake_case_variant_name: InRequestChannel<#field_type, #name #ty_generics>
        }
    });

    let match_arm = fields.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! {
            #name::#variant_name(_) => {
                stringify!(#variant_name).to_string()
            }
        }
    });

    let from_builder_inbounds = fields.iter().map(|variant| {
        let variant_name = &variant.ident;
        let snake_case_variant_name_str = variant_name.to_string().to_case(Case::Snake);
        let snake_case_variant_name = Ident::new(&snake_case_variant_name_str, variant_name.span());

        assert!(
            generics.params.len() <= 1,
            "Only zero or one generic parameter is supported, got {}",
            generics.params.len()
        );

        let field_type = if let Fields::Unnamed(fields_unnamed) = &variant.fields {
            &fields_unnamed.unnamed[0].ty
        } else {
            panic!("Enum variants must be tuples");
        };

        quote! {

            let #snake_case_variant_name = InRequestChannel::<#field_type, #name>::new(
                &mut builder.context,
                actor_name.clone(),
                &builder.request_sender,
                stringify!(#variant_name).to_owned(),
            );
            builder.forward_request.insert(
                #snake_case_variant_name.name.clone(),
                Box::new(#snake_case_variant_name.clone())
            );
        }
    });

    let from_builder_init = fields.iter().map(|variant| {
        let variant_name = &variant.ident;
        let snake_case_variant_name_str = variant_name.to_string().to_case(Case::Snake);
        let snake_case_variant_name = Ident::new(&snake_case_variant_name_str, variant_name.span());

        quote! {
            #snake_case_variant_name,
        }
    });

    let gen = quote! {
        #ast

        /// Auto-generated inbound hub for actor.
        pub struct #struct_name #impl_generics #where_clause {
            #(#in_requests),*
        }

        impl #impl_generics IsInRequestMessage for #name #ty_generics #where_clause {
            type Prop = #prop_type;
            type State = #state_type;
            type OutboundHub = #output_type;
            type OutRequestHub = #request_type;

            fn in_request_channel(&self) -> String {
                match self {
                   #(#match_arm),*
                }
            }
        }

        impl #impl_generics IsInRequestHub<
            #prop_type,
            #state_type,
            #output_type,
            #request_type,
            #message_type,
            #name #ty_generics> for #struct_name #ty_generics #where_clause
        {
            fn from_builder(
                builder: &mut ActorBuilder<
                    #prop_type,
                    #state_type,
                    #output_type,
                    #request_type,
                    #message_type,
                    #name #ty_generics,
                >,
                actor_name: &str) -> Self
            {
                #(#from_builder_inbounds)*

                #struct_name {
                    #(#from_builder_init)*
                }
            }
        }
    };

    gen.into()
}

struct ActorInRequests {
    struct_name: Ident,
    prop_type: Type,
    state_type: Type,
    output_type: Type,
    request_type: Type,
    message_type: Type,
}

impl Parse for ActorInRequests {
    fn parse(inbound: ParseStream) -> Result<Self> {
        let struct_name: Ident = inbound.parse()?;
        let _: Generics = inbound.parse()?;
        let _: Token![,] = inbound.parse()?;
        let content;
        syn::braced!(content in inbound);
        let prop_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;
        let state_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;
        let output_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;
        let request_type: Type = content.parse()?;
        let _: Token![,] = content.parse()?;
        let message_type: Type = content.parse()?;
        Ok(ActorInRequests {
            struct_name,
            prop_type,
            state_type,
            output_type,
            request_type,
            message_type,
        })
    }
}

struct ActorArgs {
    request_message_type: Type,
    message_type: Type,
}

impl Parse for ActorArgs {
    fn parse(inbound_hub: ParseStream) -> Result<Self> {
        let message_type: Type = inbound_hub.parse()?;
        let _: Token![,] = inbound_hub.parse()?;
        let request_message_type: Type = inbound_hub.parse()?;

        Ok(ActorArgs {
            message_type,
            request_message_type,
        })
    }
}

pub fn actor_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    // parse inbound
    let ActorArgs {
        message_type,
        request_message_type,
    } = match parse2::<ActorArgs>(attr) {
        Ok(args) => args,
        Err(err) => return err.to_compile_error(),
    };
    let inbound: Item = match parse2(item) {
        Ok(inbound) => inbound,
        Err(err) => return err.to_compile_error(),
    };
    let inbound_clone = inbound.clone();

    // Get actor name from the item
    let actor_name = match inbound {
        Item::Type(item) => item.ident,
        _ => panic!("`actor` attribute can only be used with type aliases"),
    };

    let mut inbound_clone = inbound_clone.clone();
    let mut attrs = Vec::new();
    if let Item::Type(item_type) = &mut inbound_clone {
        attrs.append(&mut item_type.attrs);
    }

    let mut maybe_prop = None;
    let mut maybe_inbounds = None;
    let mut maybe_in_request = None;
    let mut maybe_state = None;
    let mut maybe_outputs = None;
    let mut maybe_requests = None;

    if let Item::Type(item_type) = inbound_clone {
        if let Type::Path(type_path) = *item_type.ty {
            if type_path.path.segments.last().unwrap().ident != "Actor" {
                return Error::new_spanned(&type_path, "Expected Actor<...>")
                    .to_compile_error()
                    .into();
            }
            for segment in type_path.path.segments {
                if let PathArguments::AngleBracketed(angle_bracketed_args) = segment.arguments {
                    if angle_bracketed_args.args.len() != 6 {
                        return Error::new_spanned(
                            &angle_bracketed_args,
                            concat!(
                                "Expected 6 type arguments: ",
                                "Actor<PROP, INBOUND, INBOUND_REQUESTS, STATE,",
                                "OUTBOUND, OUTBOUND_REQUESTS>"
                            ),
                        )
                        .to_compile_error()
                        .into();
                    }
                    maybe_prop = Some(angle_bracketed_args.args[0].clone());
                    maybe_inbounds = Some(angle_bracketed_args.args[1].clone());
                    maybe_in_request = Some(angle_bracketed_args.args[2].clone());
                    maybe_state = Some(angle_bracketed_args.args[3].clone());
                    maybe_outputs = Some(angle_bracketed_args.args[4].clone());
                    maybe_requests = Some(angle_bracketed_args.args[5].clone());
                }
            }
        } else {
            return Error::new_spanned(&item_type.ty, "Expected a type path")
                .to_compile_error()
                .into();
        }
    } else {
        panic!("`actor` attribute can only be used with type aliases");
    }

    let prop = maybe_prop.unwrap();
    let inbound = maybe_inbounds.unwrap();
    let in_request = maybe_in_request.unwrap();
    let state_type = maybe_state.unwrap();
    let outbound = maybe_outputs.unwrap();
    let out_requests = maybe_requests.unwrap();

    let runner_type = quote! {
        DefaultRunner<#prop, #inbound, #in_request, #state_type,  #outbound, #out_requests>
    };

    let gen = quote! {

        #[allow(missing_docs)]
        #( #attrs )*
        pub type #actor_name =
            Actor<#prop, #inbound, #in_request, #state_type, #outbound, #out_requests>;

        impl HasFromPropState<
                #prop,
                #inbound,
                #in_request,
                #state_type,
                #outbound,
                #message_type,
                #request_message_type,
                #out_requests,
                #runner_type
            > for #actor_name
        {
            fn name_hint(prop: &#prop) -> String {
                stringify!(#actor_name).to_owned()
            }
        }
    };

    gen.into()
}
