use proc_macro2::TokenStream;
use syn::spanned::Spanned;
use syn::{Attribute, ItemStruct, LitStr};

use crate::helper_primitives::{BuildMethod, StructField, TypeOutput};

fn get_build_as_attr(attr: &Attribute) -> syn::Result<TypeOutput> {
    let attr_type = attr.parse_args::<syn::Type>();

    match attr_type {
        Ok(ty) => Ok(TypeOutput::BoxedTraitObjectType(ty)),
        Err(_) => Ok(TypeOutput::SelfType),
    }
}

fn get_build_method_attr(attr: &Attribute) -> syn::Result<BuildMethod> {
    attr.parse_args::<LitStr>()?.value().try_into()
}

pub(crate) fn expand_di_builder(input: ItemStruct) -> syn::Result<TokenStream> {
    // rules:
    // - #[build_as] is optional on the struct
    // - #[build_method] is optional on the struct
    // - #[deps] is optional on the fields
    // - #[deps] can only be used on fields and no more than once per field
    // - #[build_as] can only be used once and always before #[build_method]
    // - #[build_method] can only be used once and always after #[build_as]

    // get the #[build_as] and #[build_method] attributes according to the rules
    let (mut build_as_output, mut build_method) = (TypeOutput::SelfType, BuildMethod::None);
    let (mut build_as_output_encountered, mut build_method_encountered) = (0, 0);

    for attr in &input.attrs {
        if attr.path().is_ident("build_as") {
            build_as_output = get_build_as_attr(attr)?;
            build_as_output_encountered += 1;

            if build_as_output_encountered > 1 {
                return Err(syn::Error::new_spanned(
                    attr,
                    "Multiple #[build_as] attributes are not allowed",
                ));
            }

            if build_method_encountered > 0 {
                return Err(syn::Error::new_spanned(
                    attr,
                    "The #[build_as] attribute must come before the #[build_method] attribute",
                ));
            }
        } else if attr.path().is_ident("build_method") {
            build_method = get_build_method_attr(attr)?;
            build_method_encountered += 1;

            if build_method_encountered > 1 {
                return Err(syn::Error::new_spanned(
                    attr,
                    "Multiple #[build_method] attributes are not allowed",
                ));
            }
        }
    }

    // get the types of all fields which are annotated with #[di_build]
    let field_types = match &input.fields {
        syn::Fields::Named(fields) => fields.named.iter().collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields) => fields.unnamed.iter().collect::<Vec<_>>(),
        syn::Fields::Unit => vec![],
    }
    .iter()
    .map(|field| {
        let ty = &field.ty;

        match StructField::new(field).is_deps() {
            Ok(true) => Ok(Some(ty)),
            Ok(false) => Ok(None),
            Err(e) => Err(e),
        }
    })
    .collect::<syn::Result<Vec<_>>>()?
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let named_field_idents = match &input.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().unwrap();

                match StructField::new(field).is_deps() {
                    Ok(true) => Ok(Some(ident)),
                    Ok(false) => Ok(None),
                    Err(e) => Err(e),
                }
            })
            .collect::<syn::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    let unnamed_field_idents = match &input.fields {
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, field)| match StructField::new(field).is_deps() {
                Ok(true) => Ok(Some(syn::Ident::new(&format!("field_{}", i), field.span()))),
                Ok(false) => Ok(None),
                Err(e) => Err(e),
            })
            .collect::<syn::Result<Vec<_>>>()?
            .into_iter()
            .flatten()
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
        _ => Err(syn::Error::new_spanned(
            input.clone(),
            "Cannot mix named and unnamed fields with #[di_build]",
        ))?,
    };

    // box the build method output if the #[build_as] attribute is present
    let build_method = match build_as_output {
        TypeOutput::SelfType => build_method,
        TypeOutput::BoxedTraitObjectType(_) => quote::quote! {
            Box::new(#build_method)
        },
    };

    // get the name of the input struct
    let input_ident = &input.ident;

    let output = match (
        named_field_idents.is_empty(),
        unnamed_field_idents.is_empty(),
    ) {
        (true, true) => quote::quote! {
            #[async_trait]
            impl DIBuilder for #input_ident {
                type Input = deps!();
                type Output = #build_as_output;

                async fn build(_: Self::Input) -> Self::Output {
                    #build_method
                }
            }
        },
        (false, true) | (true, false) => quote::quote! {
            #[async_trait]
            impl DIBuilder for #input_ident {
                type Input = deps!(#(#field_types),*);
                type Output = #build_as_output;

                async fn build(input: Self::Input) -> Self::Output {
                    #build_method
                }
            }
        },
        _ => Err(syn::Error::new_spanned(
            input,
            "Cannot mix named and unnamed fields with #[di_build]",
        ))?,
    };

    Ok(output)
}
