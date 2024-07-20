use crate::core::primitives::DIManager;
use async_trait::async_trait;

/// A trait for building dependencies.
///
/// The `DIBuilder` trait is used to define a dependency that can be built by the dependency injection manager.
/// For each dependency, you need to implement the [`build`](DIBuilder::build) method that takes
/// as input the [`Self::Input`](DIBuilder::Input) associated type and returns the [`Self::Output`](DIBuilder::Output) associated type.
#[async_trait]
pub trait DIBuilder {
    /// The input type embedding all the dependencies needed to build the current dependency.
    ///
    /// The input type is used to pass dependencies to the builder. Keep in mind that the input type
    /// must implement the [`GetInput`](GetInput) trait to be able to retrieve its own dependencies
    /// from the dependency injection manager if any.
    ///
    /// As [`GetInput`](GetInput) is using a recursive implementation via inductive traits, you can use tuples to pass multiple dependencies.
    /// Also, you should wrap each dependency in a [`DIObj`](super::primitives::DIObj) to ensure thread safety and that the dependency can be easily
    /// resolved by the dependency injection manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use yadir::{deps, let_deps};
    /// use yadir::core::contracts::{DIBuilder, GetInput};
    /// use yadir::core::primitives::{DIManager, DIObj};
    /// use yadir_derive::DIBuilder;
    /// 
    /// #[derive(Clone, DIBuilder)]
    /// struct Bar;
    /// 
    /// #[derive(Clone, DIBuilder)]
    /// struct Foo(#[deps] Bar);
    /// 
    /// # #[tokio::main]
    /// # async fn main() {
    /// #    let mut manager = DIManager::default();
    /// #
    /// #    manager.build::<Bar>().await;
    /// #    manager.build::<Foo>().await;
    /// #
    /// #    assert!(manager.has::<DIObj<Bar>>());
    /// # }
    /// ```
    type Input: GetInput + Clone;

    /// The output type representing the built dependency.
    ///
    /// The output type is the type of the dependency that will be built by the builder after resolving all its dependencies.
    /// Notice that the lifetime of the output type must be `'static` to ensure that the dependency injection manager does not
    /// allow for invalid references to types to be stored in the type map.
    type Output: 'static + Clone;

    /// Builds the dependency using the input type.
    ///
    /// The `build` method is used to build the dependency using the input type. The input type is used to pass all the dependencies
    /// needed to build the current dependency. The method should return the built dependency as the output type.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use yadir::{deps, let_deps};
    /// use yadir::core::contracts::{DIBuilder, GetInput};
    /// use yadir::core::primitives::{DIManager, DIObj};
    /// use yadir_derive::DIBuilder;
    /// 
    /// #[derive(Clone, DIBuilder)]
    /// struct Bar;
    /// 
    /// #[derive(Clone, DIBuilder)]
    /// struct Foo(#[deps] Bar);
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///    let mut manager = DIManager::default();
    ///
    ///    manager.build::<Bar>().await;
    ///    manager.build::<Foo>().await;
    ///
    ///    assert!(manager.has::<DIObj<Bar>>());
    /// }
    /// ```
    async fn build(input: Self::Input) -> Self::Output;
}

/// A trait used to retrieve dependencies from the dependency injection manager.
///
/// The `GetInput` trait is used to inductively resolve all the dependencies needed to build the implementer type from the dependency injection manager.
/// It is implemented for the following cases:
/// - [`DIObj<T>`](super::primitives::DIObj): to retrieve a dependency wrapped in a thread-safe reference counted mutex from the dependency injection manager (**base case**).
/// - `()`: to return the unit type when no dependencies are needed (**base case**).
/// - `(S, T)`: to retrieve multiple dependencies by recursively resolving each dependency (**inductive case**).
pub trait GetInput: Sized {
    fn get_input(manager: &DIManager) -> Option<Self>;
}
