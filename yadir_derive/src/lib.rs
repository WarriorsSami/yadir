enum BuildMethod {
    None,
    New,
    Default,
}

impl From<String> for BuildMethod {
    fn from(s: String) -> Self {
        match s.as_str() {
            "new" => Self::New,
            "default" => Self::Default,
            _ => Self::None,
        }
    }
}

#[proc_macro_derive(DIBuilder, attributes(deps, build_method))]
pub fn derive_di_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemStruct);
    
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
        .map(BuildMethod::from)
        .unwrap_or(BuildMethod::None);

    // get the types of all fields which are annotated with #[di_build]
    let field_types = match &input.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .filter_map(|field| {
                let ty = &field.ty;

                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("deps"))
                {
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

                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("deps"))
                {
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

                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("deps"))
                {
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

                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident("deps"))
                {
                    Some(ident)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        _ => vec![],
    };
    
    // construct the instantiation of the input struct based on the #[deps] fields and the #[build_method] attribute
    let build_method = match (build_method, named_field_idents.is_empty(), unnamed_field_idents.is_empty()) {
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
        (BuildMethod::Default, false, true) => quote::quote! {
            Self::default()
        },
        (BuildMethod::Default, true, false) => quote::quote! {
            Self::default()
        },
        _ => panic!("Cannot mix named and unnamed fields with #[di_build]"),
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
                type Output = Self;

                async fn build(_: Self::Input) -> Self::Output {
                    #build_method
                }
            }
        },
        (false, true) | (true, false) => quote::quote! {
            #[async_trait]
            impl DIBuilder for #input {
                type Input = deps!(#(#field_types),*);
                type Output = Self;

                async fn build(input: Self::Input) -> Self::Output {
                    #build_method
                }
            }
        },
        _ => panic!("Cannot mix named and unnamed fields with #[di_build]"),
    };

    output.into()
}
