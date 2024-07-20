use proc_macro2::TokenStream;

pub(crate) enum BuildMethod {
    None,
    New,
    Default,
}

impl TryFrom<String> for BuildMethod {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "new" => Ok(Self::New),
            "default" => Ok(Self::Default),
            _ => Err("Invalid build method".into()),
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

    pub(crate) fn is_deps(&self) -> bool {
        self.field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("deps"))
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
