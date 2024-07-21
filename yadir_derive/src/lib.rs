//! # yadir_derive
//!
//! This crate provides helpful procedural macros for the `yadir` crate.

use crate::expand_handlers::expand_di_builder;
use proc_macro_error::proc_macro_error;

mod expand_handlers;
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
///    - `None` (the attribute is missing): Directly instantiates the input struct.
/// - `#[deps]`: Specifies the fields that are input dependencies for the builder.
/// 
/// Rules for attributes usage:
/// - `#[build_as]` is optional on the struct
/// - `#[build_method]` is optional on the struct
/// - `#[deps]` is optional on the fields
/// - `#[deps]` can only be used on fields and no more than once per field
/// - `#[build_as]` can only be used once and always before `#[build_method]`
/// - `#[build_method]` can only be used once and always after `#[build_as]`
///
/// # Example
///
/// ```ignore
///
/// trait Printer: Sync + Send + DynClone {
///    fn print(&self) -> String;
/// }
///
/// #[derive(Clone, DIBuilder)]
/// #[build_as(Box<dyn Printer>)]
/// struct Bar;
///
/// impl Printer for Bar {
///     fn print(&self) -> String {
///         "bar".to_string()
///     }
/// }
///
/// #[derive(Default, Clone, DIBuilder)]
/// #[build_method("default")]
/// struct Baz;
///
/// #[derive(Clone, DIBuilder)]
/// #[build_method("new")]
/// struct Qux;
///
/// impl Qux {
///    pub fn new() -> Self {
///       Self
///   }
/// }
///
/// #[derive(Clone, DIBuilder)]
/// #[build_method("new")]
/// struct Foo {
///    #[deps]
///    bar: Box<dyn Printer>,
///    #[deps]
///    baz: Baz,
///    #[deps]
///    qux: Qux,
/// }
///
/// impl Foo {
///     pub fn new(bar: Box<dyn Printer>, baz: Baz, qux: Qux) -> Self {
///        Self { bar, baz, qux }
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_derive(DIBuilder, attributes(build_as, build_method, deps))]
pub fn derive_di_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemStruct);
    expand_di_builder(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
