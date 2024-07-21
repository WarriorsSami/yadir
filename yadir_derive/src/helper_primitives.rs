use proc_macro2::TokenStream;

pub(crate) enum BuildMethod {
    None,
    New,
    Default,
}

impl TryFrom<String> for BuildMethod {
    type Error = syn::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "new" => Ok(Self::New),
            "default" => Ok(Self::Default),
            _ => Err(syn::Error::new_spanned(value, "Invalid build method")),
        }
    }
}

pub(crate) struct StructField<'f> {
    field: &'f syn::Field,
}

impl<'f> StructField<'f> {
    pub(crate) fn new(field: &'f syn::Field) -> Self {
        Self { field }
    }

    pub(crate) fn is_deps(&self) -> syn::Result<bool> {
        let deps_attrs = self
            .field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("deps"))
            .collect::<Vec<_>>();

        match deps_attrs.len() {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(syn::Error::new_spanned(
                deps_attrs[1].path(),
                "Multiple #[deps] attributes are redundant",
            )),
        }
    }
}

pub(crate) enum TypeOutput {
    SelfType,
    BoxedTraitObjectType(syn::Type),
}

impl quote::ToTokens for TypeOutput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::SelfType => quote::quote! {
                Self
            },
            Self::BoxedTraitObjectType(ty) => quote::quote! {
                #ty
            },
        }
        .to_tokens(tokens)
    }
}
