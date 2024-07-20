use crate::core::contracts::{DIBuilder, GetInput};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A simple type map that stores values by their type.
#[derive(Default)]
pub struct TypeMap(HashMap<TypeId, Box<dyn Any>>);

impl TypeMap {
    /// Inserts a value into the map with its inferred type as the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use yadir::core::primitives::TypeMap;
    ///
    /// let mut map = TypeMap::default();
    /// map.set(42);
    ///
    /// assert_eq!(map.get::<i32>(), Some(&42));
    /// ```
    pub fn set<T>(&mut self, t: T)
    where
        T: Any + 'static,
    {
        self.0.insert(TypeId::of::<T>(), Box::new(t));
    }

    /// Retrieves a value from the map by its type. Returns `None` if the value is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use yadir::core::primitives::TypeMap;
    ///
    /// let mut map = TypeMap::default();
    ///
    /// assert_eq!(map.get::<i32>(), None);
    /// ```
    pub fn get<T>(&self) -> Option<&T>
    where
        T: Any + 'static,
    {
        self.0
            .get(&TypeId::of::<T>())
            .map(|boxed| boxed.downcast_ref::<T>().unwrap())
    }

    /// Retrieves a mutable reference to a value from the map by its type. Returns `None` if the value is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use yadir::core::primitives::TypeMap;
    ///
    /// let mut map = TypeMap::default();
    /// map.set(42);
    ///
    /// assert_eq!(map.get_mut::<i32>(), Some(&mut 42));
    ///
    /// let mut value = map.get_mut::<i32>().unwrap();
    /// *value = 43;
    ///
    /// assert_eq!(map.get::<i32>(), Some(&43));
    /// ```
    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: Any + 'static,
    {
        self.0
            .get_mut(&TypeId::of::<T>())
            .map(|boxed| boxed.downcast_mut::<T>().unwrap())
    }

    /// Checks if the map contains a value of a given type.
    ///
    /// # Examples
    ///
    /// ```
    /// use yadir::core::primitives::TypeMap;
    ///
    /// let mut map = TypeMap::default();
    /// map.set(42);
    ///
    /// assert!(map.has::<i32>());
    /// assert!(!map.has::<String>());
    /// ```
    pub fn has<T>(&self) -> bool
    where
        T: Any + 'static,
    {
        self.0.contains_key(&TypeId::of::<T>())
    }
}

/// A new type wrapper for a thread-safe reference counted mutex to handle thread-safe sharing of embedded dependencies.
#[derive(Clone)]
pub struct DIObj<T: Clone>(Arc<Mutex<T>>);

impl<T: Clone> DIObj<T> {
    pub fn new(t: T) -> Self {
        Self(Arc::new(Mutex::new(t)))
    }

    pub fn extract(&self) -> T {
        self.0.lock().unwrap().clone()
    }
}

/// A struct used to model a dependency injection manager.
///
/// The `DIManager` struct is used to manage the dependencies and build them using the [`build`](DIManager::build) method.
/// The manager uses a [`TypeMap`](TypeMap) to store the dependencies by their type.
#[derive(Default)]
pub struct DIManager(TypeMap);

impl DIManager {
    /// Builds a dependency using the dependency injection manager.
    ///
    /// The `build` method is used to build a dependency using the dependency injection manager. The method takes a type parameter `T`
    /// that must implement the [`DIBuilder`](DIBuilder) trait. Afterward, it returns a [`DIObj`](DIObj) that wraps the built dependency
    /// and stores it in the dependency injection manager.
    ///
    /// The method returns `None` if the dependency could not be built.
    ///
    /// # Examples
    ///
    /// ```
    /// # use async_trait::async_trait;
    /// # use yadir::{deps, let_deps};
    /// # use yadir::core::contracts::{DIBuilder};
    /// # use yadir::core::primitives::DIObj;
    /// use yadir::core::primitives::DIManager;
    /// # use yadir_derive::DIBuilder;
    ///
    /// # #[derive(Clone, DIBuilder)]
    /// # struct Bar;
    /// #
    /// # #[derive(Clone, DIBuilder)]
    /// # struct Foo(#[deps] Bar);
    /// #
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut manager = DIManager::default();
    ///
    ///     manager.build::<Bar>().await;
    ///     manager.build::<Foo>().await;
    ///
    ///     assert!(manager.has::<DIObj<Bar>>());
    /// }
    /// ```
    pub async fn build<T>(&mut self) -> Option<DIObj<T::Output>>
    where
        T: DIBuilder,
    {
        let input = T::Input::get_input(self)?;
        let obj = T::build(input).await;
        let sync_obj = DIObj::new(obj);
        self.0.set::<DIObj<T::Output>>(sync_obj.clone());
        Some(sync_obj)
    }

    /// Checks if the dependency injection manager contains a dependency of a given type.
    ///
    /// The `has` method is used to check if the dependency injection manager contains a dependency of a given type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use async_trait::async_trait;
    /// # use yadir::{deps, let_deps};
    /// # use yadir::core::contracts::{DIBuilder};
    /// # use yadir::core::primitives::{DIObj};
    /// use yadir::core::primitives::DIManager;
    /// # use yadir_derive::DIBuilder;
    ///
    /// # #[derive(Clone, DIBuilder)]
    /// # struct Bar;
    /// #
    /// # #[derive(Clone, DIBuilder)]
    /// # struct Foo(#[deps] Bar);
    /// #
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut manager = DIManager::default();
    ///
    ///     manager.build::<Bar>().await;
    /// #   manager.build::<Foo>().await;
    ///
    ///     assert!(manager.has::<DIObj<Bar>>());
    /// }
    /// ```
    pub fn has<T>(&self) -> bool
    where
        T: Any + 'static,
    {
        self.0.has::<T>()
    }
}

impl<T: Clone + 'static> GetInput for DIObj<T> {
    fn get_input(manager: &DIManager) -> Option<Self> {
        manager.0.get::<Self>().cloned()
    }
}

impl GetInput for () {
    fn get_input(_: &DIManager) -> Option<Self> {
        Some(())
    }
}

impl<S, T> GetInput for (S, T)
where
    S: GetInput,
    T: GetInput,
{
    fn get_input(manager: &DIManager) -> Option<Self> {
        S::get_input(manager).and_then(|s| T::get_input(manager).map(|t| (s, t)))
    }
}
