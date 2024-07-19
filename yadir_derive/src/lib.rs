#[proc_macro_derive(DIBuilder, attributes(di_build))]
pub fn derive_di_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemStruct);

    // get the types of all fields which are annotated with #[di_build]
    let field_types = match &input.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let ty = &field.ty;
                quote::quote! {
                    #ty
                }
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(_, field)| {
                let ty = &field.ty;
                quote::quote! {
                    #ty
                }
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unit => vec![],
    };

    let field_idents = match &input.fields {
        syn::Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let ident = field.ident.as_ref().unwrap();
                quote::quote! {
                    #ident
                }
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let ident = syn::Ident::new(&format!("field{}", i), proc_macro2::Span::call_site());
                quote::quote! {
                    #ident
                }
            })
            .collect::<Vec<_>>(),
        syn::Fields::Unit => vec![],
    };

    // get the name of the input struct
    let input = input.ident;

    let output = if field_idents.is_empty() {
        quote::quote! {
            #[async_trait]
            impl DIBuilder for #input {
                type Input = deps!();
                type Output = Self;

                async fn build(_: Self::Input) -> Self::Output {
                    Self
                }
            }
        }
    } else {
        quote::quote! {
            #[async_trait]
            impl DIBuilder for #input {
                type Input = deps!(#(#field_types),*);
                type Output = Self;

                async fn build(input: Self::Input) -> Self::Output {
                    let_deps!(#(#field_idents),* <- input);

                    Self {
                        #(
                            #field_idents
                        ),*
                    }
                }
            }
        }
    };

    output.into()
}
