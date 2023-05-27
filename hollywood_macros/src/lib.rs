#![deny(missing_docs)]

//! Convenience macros for the Hollywood actor framework.

extern crate proc_macro;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, Error, Fields, Ident, Item, ItemEnum,
    ItemStruct, Path, PathArguments, Result, Token, Type, TypePath,
};

struct ActorArgs {
    message_type: Ident,
}

impl Parse for ActorArgs {
    fn parse(inbound: ParseStream) -> Result<Self> {
        let message_type: Ident = inbound.parse()?;
        Ok(ActorArgs { message_type })
    }
}

/// This macro generates the boilerplate to define an new actor type.
#[proc_macro_attribute]
pub fn actor(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse inbound
    let ActorArgs { message_type } = parse_macro_input!(attr as ActorArgs);
    let inbound: Item = parse_macro_input!(item);
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
        // ...
    }

    let mut maybe_inbounds = None;
    let mut maybe_state = None;
    let mut maybe_outputs = None;
    if let Item::Type(item_type) = inbound_clone {
        if let Type::Path(type_path) = *item_type.ty {
            if type_path.path.segments.last().unwrap().ident != "Actor" {
                return Error::new_spanned(&type_path, "Expected Actor<...>")
                    .to_compile_error()
                    .into();
            }
            for segment in type_path.path.segments {
                if let PathArguments::AngleBracketed(angle_bracketed_args) = segment.arguments {
                    if angle_bracketed_args.args.len() != 3 {
                        return Error::new_spanned(
                            &angle_bracketed_args,
                            "Expected three type arguments: Actor<inboundS, STATE, OUTPUTS>",
                        )
                        .to_compile_error()
                        .into();
                    }
                    maybe_inbounds = Some(angle_bracketed_args.args[0].clone());
                    maybe_state = Some(angle_bracketed_args.args[1].clone());
                    maybe_outputs = Some(angle_bracketed_args.args[2].clone());
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

    let inbound = maybe_inbounds.unwrap();
    let state_type = maybe_state.unwrap();
    let out = maybe_outputs.unwrap();

    let runner_type = quote! { DefaultRunner<#inbound, #state_type,  #out> };

    let gen = quote! {

        ///
        #( #attrs )*
        pub type #actor_name = Actor<#inbound, #state_type, #out>;

        impl DynActor< #inbound, #state_type, #out, #message_type, #runner_type>
            for #actor_name
        {
            fn name_hint() -> String {
                stringify!(#actor_name).to_owned()
            }
        }
    };

    gen.into()
}

struct ActorInbounds {
    struct_name: Ident,
    state_type: Ident,
    output_type: Ident,
}

impl Parse for ActorInbounds {
    fn parse(inbound: ParseStream) -> Result<Self> {
        let struct_name: Ident = inbound.parse()?;
        let _: Token![,] = inbound.parse()?;
        let content;
        syn::braced!(content in inbound);
        let state_type: Ident = content.parse()?;
        let _: Token![,] = content.parse()?;
        let output_type: Ident = content.parse()?;
        Ok(ActorInbounds {
            struct_name,
            state_type,
            output_type,
        })
    }
}

/// This macro generates the boilerplate for the inbound reception of an actor.
#[proc_macro_attribute]
pub fn actor_inputs(args: TokenStream, inbound: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as ActorInbounds);
    let ast = parse_macro_input!(inbound as ItemEnum);

    let name = &ast.ident;
    let fields = &ast.variants;

    let struct_name = &args.struct_name;
    let state_type = &args.state_type;
    let output_type = &args.output_type;

    let inbound = fields.iter().map(|variant| {
        let variant_name = variant.ident.clone();
        let snake_case_variant_name = variant_name.to_string().to_case(Case::Snake);
        let snake_case_variant_name = Ident::new(&snake_case_variant_name, variant_name.span());
        let field_type = if let Fields::Unnamed(fields_unnamed) = &variant.fields {
            &fields_unnamed.unnamed[0].ty
        } else {
            panic!("Enum variants must be tuples");
        };

        let msg = format!(
            "Autogenerated field for `{}` channel by the `actor_inputs` macro.",
            variant_name
        );
        quote! {
            #[doc = #msg]
            pub  #snake_case_variant_name: InboundChannel<#field_type, #name>
        }
    });

    let match_arm = fields.iter().map(|variant| {
        let variant_name = &variant.ident;
        if let Fields::Unnamed(fields_unnamed) = &variant.fields {
            &fields_unnamed.unnamed[0].ty
        } else {
            panic!("Enum variants must be tuples");
        };

        quote! {
            #name::#variant_name(msg) => {
                stringify!(#variant_name).to_string()
            }
        }
    });

    let all_inbound_types = fields
        .iter()
        .map(|variant| {
            if let Fields::Unnamed(fields_unnamed) = &variant.fields {
                let field_type = &fields_unnamed.unnamed[0].ty;
                quote! { InboundChannel<#field_type, #name> }
            } else {
                panic!("Enum variants must be tuples");
            }
        })
        .collect::<Vec<_>>();

    let all_inbounds_type = quote! { ( #(#all_inbound_types),* ) };

    let from_builder_inbounds = fields.iter().map(|variant| {
        let variant_name = &variant.ident;
        let snake_case_variant_name = variant.ident.clone().to_string().to_case(Case::Snake);
        let snake_case_variant_name = Ident::new(&snake_case_variant_name, variant_name.span());
        let field_type = if let Fields::Unnamed(fields_unnamed) = &variant.fields {
            &fields_unnamed.unnamed[0].ty
        } else {
            panic!("Enum variants must be tuples");
        };

        quote! {
            let #snake_case_variant_name = InboundChannel::<#field_type, #name>::new(
                &mut builder.context,
                actor_name.clone(),
                &builder.sender,
                #name::#variant_name(Default::default()).inbound_name(),
            );
            builder
                .forward
                .insert(#snake_case_variant_name.name.clone(), Box::new(#snake_case_variant_name.clone()));
        }
    });

    let from_builder_init = fields.iter().map(|variant| {
        let variant_name = variant.ident.clone();
        let snake_case_variant_name = variant_name.to_string().to_case(Case::Snake);
        let snake_case_variant_name = Ident::new(&snake_case_variant_name, variant_name.span());

        quote! {
            #snake_case_variant_name,
        }
    });

    let gen = quote! {
        #ast

        /// Auto-generated inbound reception for actor.
        pub struct #struct_name {
            #(#inbound),*
        }

        impl InboundMessage for #name {
            type State = #state_type;
            type OutboundDistribution = #output_type;

            fn inbound_name(&self) -> String {
                match self {
                   #(#match_arm),*
                }
            }
        }

        impl InboundReceptionTrait<#state_type, #output_type, #name> for #struct_name {
            type AllInbounds = #all_inbounds_type;

            fn from_builder(builder: &mut ActorBuilder<#state_type, #output_type, #name>,
                            actor_name: &str) -> Self {
                #(#from_builder_inbounds)*

                #struct_name {
                    #(#from_builder_init)*
                }
            }
        }

    };

    gen.into()
}

// This function check if the field's type is OutboundChannel<T> and return T if it is
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

/// This macro generates the boilerplate for the outbound distribution center of an actor.
#[proc_macro_attribute]
pub fn actor_outputs(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as ItemStruct);

    let struct_name = &ast.ident;
    let fields = match &ast.fields {
        Fields::Named(fields_named) => &fields_named.named,
        _ => panic!("`generate_outputs_trait` can only be used with structs with named fields"),
    };

    let output_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        if let Some(inner_ty) = is_output_type(&field.ty) {
            // if the field type is OutboundChannel<T>, use OutboundChannel::<T>
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
        #ast

        impl OutboundDistributionTrait for #struct_name {
            fn from_context_and_parent(context: &mut Context, actor_name: &str) -> Self {
                Self {
                    #(#output_assignments),*
                }
            }
        }

        impl Morph for #struct_name {
            fn extract(&mut self) -> Self {
                Self {
                    #(#output_extract),*
                }
            }

            fn activate(&mut self) {
                #(#output_act)*
            }
        }

    };

    gen.into()
}
