//! # yadir_derive
//! 
//! This crate provides helpful procedural macros for the `yadir` crate.

use crate::helper_primitives::{BuildMethod, StructField, TypeOutput};

mod helper_primitives;

/// Derive the `DIBuilder` trait for a struct.
/// 
/// This proc macro is used to automatically derive the `DIBuilder` trait for a struct. 
/// The `DIBuilder` trait is used to build a dependency by specifying the input/dependencies, the output, and the build method for a given dependency.
/// 
/// The `#[derive(DIBuilder)]` macro provides a few helper attributes to customize the behavior of the builder:
/// - `#[build_as]`: Specifies the output type of the builder. If this attribute is not present, the output type will be the input struct itself. 
/// - `#[build_method]`: Specifies the method to build the dependency, which can be one of the following:
///    - `new`: Calls the `new` method on the input struct.
///    - `default`: Calls the `Default` trait implementation for the input struct.
///    - `None`: Directly instantiates the input struct.
/// - `#[deps]`: Specifies the fields that are input dependencies for the builder.
#[proc_macro_derive(DIBuilder, attributes(build_as, build_method, deps))]
pub fn derive_di_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemStruct);

    // get the value of the #[build_as] attribute as a type
    let build_as_output = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("build_as"))
        .map(|attr| TypeOutput::BoxedTraitObjectType(attr.parse_args::<syn::Type>().expect("expected a type")))
        .unwrap_or_else(|| TypeOutput::SelfType);

    // get the value of the #[build_method] attribute
    let build_method = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("build_method"))
        .map(|attr| {
            attr.parse_args::<syn::LitStr>()
                .expect("expected a string literal")
                .value()
        })
        .map(BuildMethod::try_from)
        .transpose()
        .expect("failed to parse build method")
        .unwrap_or(BuildMethod::None);

    // get the types of all fields which are annotated with #[di_build]
    let field_types = match &input.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .filter_map(|field| {
                let ty = &field.ty;

                if StructField::new(field).is_deps() {
                    Some(quote::quote! {
                        #ty
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .filter_map(|(_, field)| {
                let ty = &field.ty;

                if StructField::new(field).is_deps() {
                    Some(quote::quote! {
                        #ty
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unit => vec![],
    };

    let named_field_idents = match &input.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .filter_map(|field| {
                let ident = field.ident.as_ref().unwrap();

                if StructField::new(field).is_deps() {
                    Some(ident)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    let unnamed_field_idents = match &input.fields {
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .filter_map(|(i, field)| {
                let ident =
                    syn::Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site());

                if StructField::new(field).is_deps() {
                    Some(ident)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    // construct the instantiation of the input struct based on the #[deps] fields and the #[build_method] attribute
    let build_method = match (
        build_method,
        named_field_idents.is_empty(),
        unnamed_field_idents.is_empty(),
    ) {
        (BuildMethod::None, true, true) => quote::quote! {
            Self
        },
        (BuildMethod::None, false, true) => quote::quote! {
            let_deps!(#(#named_field_idents),* <- input);

            Self {
                #(
                    #named_field_idents
                ),*
            }
        },
        (BuildMethod::None, true, false) => quote::quote! {
            let_deps!(#(#unnamed_field_idents),* <- input);

            Self(
                #(
                    #unnamed_field_idents
                ),*
            )
        },
        (BuildMethod::New, true, true) => quote::quote! {
            Self::new()
        },
        (BuildMethod::New, false, true) => quote::quote! {
            let_deps!(#(#named_field_idents),* <- input);

            Self::new(
                #(
                    #named_field_idents
                ),*
            )
        },
        (BuildMethod::New, true, false) => quote::quote! {
            let_deps!(#(#unnamed_field_idents),* <- input);

            Self::new(
                #(
                    #unnamed_field_idents
                ),*
            )
        },
        (BuildMethod::Default, false, true) | (BuildMethod::Default, true, false) => {
            quote::quote! {
                Self::default()
            }
        }
        _ => panic!("Cannot mix named and unnamed fields with #[di_build]"),
    };
    
    // box the build method output if the #[build_as] attribute is present
    let build_method = match build_as_output {
        TypeOutput::SelfType => build_method,
        TypeOutput::BoxedTraitObjectType(_) => quote::quote! {
            Box::new(#build_method)
        },
    };

    // get the name of the input struct
    let input = input.ident;

    let output = match (
        named_field_idents.is_empty(),
        unnamed_field_idents.is_empty(),
    ) {
        (true, true) => quote::quote! {
            #[async_trait]
            impl DIBuilder for #input {
                type Input = deps!();
                type Output = #build_as_output;

                async fn build(_: Self::Input) -> Self::Output {
                    #build_method
                }
            }
        },
        (false, true) | (true, false) => quote::quote! {
            #[async_trait]
            impl DIBuilder for #input {
                type Input = deps!(#(#field_types),*);
                type Output = #build_as_output;

                async fn build(input: Self::Input) -> Self::Output {
                    #build_method
                }
            }
        },
        _ => panic!("Cannot mix named and unnamed fields with #[di_build]"),
    };

    output.into()
}
