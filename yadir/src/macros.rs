/// A helper macro used to define dependencies for a given type as nested tuples of [`DIObj`](super::core::primitives::DIObj).
///
/// First of all, you need to define the dependencies for a given type using the `deps!` macro:
///
/// ```ignore
/// deps!(Bar, Baz, Qux);
/// ```
///
/// Next, it will expand into a nested tuple of [`DIObj`](super::core::primitives::DIObj) for each dependency:
///
/// ```ignore
/// (DIObj<Bar>, (DIObj<Baz>, (DIObj<Qux>, ())))
/// ```
#[macro_export]
macro_rules! deps {
    () => {
        ()
    };
    ($t:ty) => {
        (DIObj<$t>, ())
    };
    ($t:ty, $($rest:ty),*) => {
        (DIObj<$t>, deps!($($rest),*))
    };
}

/// A helper macro used to match dependencies for a given type as nested tuples of [`DIObj`](super::core::primitives::DIObj).
///
/// The `deps_match!` macro is used to pattern match the dependencies for a given type as nested tuples of [`DIObj`](super::core::primitives::DIObj).
///
/// For example, for the following specified dependencies:
///
/// ```ignore
/// type Input = deps!(Bar, Baz, Qux);
/// ```
///
/// You can use the `deps_match!` macro to pattern match the dependencies as follows:
///
/// ```ignore
/// let input: Input;
/// let_deps!(bar, baz, qux <- input);
/// ```
#[macro_export]
macro_rules! let_deps {
    ($name:ident <- $input:ident) => {
        let $name = $input.0.extract();
    };
    ($name:ident, $( $rest:ident )* <- $input:ident) => {
        let $name = $input.0.extract();
        let $input = $input.1;
        let_deps!($( $rest )* <- $input);
    };
}
