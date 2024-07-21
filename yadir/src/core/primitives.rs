use crate::core::contracts::{DIBuilder, GetInput, GetInputKeys};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A simple enum to represent the lifetime of a dependency.
///
/// The `Lifetime` enum is used to represent the lifetime of a dependency. The enum has two variants:
/// - `Transient`: Represents a dependency that is created each time it is requested.
/// - `Singleton`: Represents a dependency that is created once and shared across all requests.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Lifetime {
    #[default]
    Transient,
    Singleton,
}

/// A simple struct to represent a key for a dependency.
///
/// The `Key` struct is used to represent a key for a dependency. The struct has two fields:
/// - `type_id`: Represents the type of the dependency.
/// - `code`: Represents an optional string code for the dependency to differentiate between multiple keyed dependencies of the same type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Key {
    pub(crate) type_id: TypeId,
    pub(crate) code: String,
}

impl Key {
    pub(crate) fn new<T>(code: String) -> Self
    where
        T: Any + 'static,
    {
        Self {
            type_id: TypeId::of::<T>(),
            code,
        }
    }

    pub(crate) fn new_with_default_code<T>() -> Self
    where
        T: Any + 'static,
    {
        Self::new::<T>(String::from("default"))
    }
}

/// A simple type map that stores values by their type and/or key.
#[derive(Default)]
pub struct TypeMap(HashMap<Key, (Lifetime, Box<dyn Any>)>);

impl TypeMap {
    /// Creates a new key for a given type based on the generic type parameter and an optional string code.
    fn get_key<T>(code: Option<String>) -> Key
    where
        T: Any + 'static,
    {
        match code {
            Some(code) => Key::new::<T>(code),
            None => Key::new_with_default_code::<T>(),
        }
    }

    /// Inserts a value into the map with its inferred type as the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use yadir::core::primitives::TypeMap;
    ///
    /// let mut map = TypeMap::default();
    /// map.set(42, None, None);
    ///
    /// assert_eq!(map.get::<i32>(None), Some(&42));
    /// ```
    pub fn set<T>(&mut self, t: T, lifetime: Option<Lifetime>, code: Option<String>)
    where
        T: Any + 'static,
    {
        self.0.insert(
            Self::get_key::<T>(code),
            (lifetime.unwrap_or_default(), Box::new(t)),
        );
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
    /// assert_eq!(map.get::<i32>(None), None);
    /// ```
    pub fn get<T>(&self, code: Option<String>) -> Option<&T>
    where
        T: Any + 'static,
    {
        self.0
            .get(&Self::get_key::<T>(code))
            .map(|(_, boxed)| boxed.downcast_ref::<T>().unwrap())
    }

    /// Retrieves a mutable reference to a value from the map by its type. Returns `None` if the value is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use yadir::core::primitives::TypeMap;
    ///
    /// let mut map = TypeMap::default();
    /// map.set(42, None, None);
    ///
    /// assert_eq!(map.get_mut::<i32>(None), Some(&mut 42));
    ///
    /// let mut value = map.get_mut::<i32>(None).unwrap();
    /// *value = 43;
    ///
    /// assert_eq!(map.get::<i32>(None), Some(&43));
    /// ```
    pub fn get_mut<T>(&mut self, code: Option<String>) -> Option<&mut T>
    where
        T: Any + 'static,
    {
        self.0
            .get_mut(&Self::get_key::<T>(code))
            .map(|(_, boxed)| boxed.downcast_mut::<T>().unwrap())
    }

    /// Retrieves the lifetime of a value from the map by its type. Returns `None` if the value is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use yadir::core::primitives::{Lifetime, TypeMap};
    ///
    /// let mut map = TypeMap::default();
    ///
    /// assert_eq!(map.get_lifetime::<i32>(None), None);
    ///
    /// map.set(42, Some(Lifetime::Singleton), None);
    /// assert_eq!(map.get_lifetime::<i32>(None), Some(Lifetime::Singleton));
    /// ```
    pub fn get_lifetime<T>(&self, code: Option<String>) -> Option<Lifetime>
    where
        T: Any + 'static,
    {
        self.0
            .get(&Self::get_key::<T>(code))
            .map(|(lifetime, _)| *lifetime)
    }

    /// Checks if the map contains a value of a given type.
    ///
    /// # Examples
    ///
    /// ```
    /// use yadir::core::primitives::TypeMap;
    ///
    /// let mut map = TypeMap::default();
    /// map.set(42, None, Some(String::from("my_key")));
    ///
    /// assert!(map.has::<i32>(Some(String::from("my_key"))));
    /// assert!(!map.has::<String>(None));
    /// ```
    pub fn has<T>(&self, code: Option<String>) -> bool
    where
        T: Any + 'static,
    {
        self.0.contains_key(&Self::get_key::<T>(code))
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
        let input = T::Input::get_input(self, 0)?;
        let obj = T::build(input).await;
        let sync_obj = DIObj::new(obj);
        self.0
            .set::<DIObj<T::Output>>(sync_obj.clone(), Some(Lifetime::Transient), None);

        Some(sync_obj)
    }

    /// Registers a dependency using the dependency injection manager with an optional lifetime and returns a mutable reference to the manager allowing for further chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use yadir::{deps, let_deps};
    /// use yadir::core::contracts::{DIBuilder};
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
    ///     let mut manager = DIManager::default();
    ///
    ///     manager
    ///         .register::<Bar>(None).await
    ///         .register::<Foo>(None).await;
    ///
    ///     assert!(manager.has::<DIObj<Bar>>());
    ///     assert!(manager.has::<DIObj<Foo>>());
    /// }
    /// ```
    pub async fn register<T>(&mut self, lifetime: Option<Lifetime>) -> &mut Self
    where
        T: DIBuilder,
    {
        let input = T::Input::get_input(self, 0)
            .expect("Some input dependencies are missing. Please register them beforehand.");
        let obj = T::build(input).await;
        let sync_obj = DIObj::new(obj);
        self.0
            .set::<DIObj<T::Output>>(sync_obj.clone(), lifetime, None);

        self
    }

    pub async fn register_with_key<T>(
        &mut self,
        lifetime: Option<Lifetime>,
        key: String,
    ) -> &mut Self
    where
        T: DIBuilder,
    {
        let input = T::Input::get_input(self, 0)
            .expect("Some input dependencies are missing. Please register them beforehand.");
        let obj = T::build(input).await;
        let sync_obj = DIObj::new(obj);
        self.0
            .set::<DIObj<T::Output>>(sync_obj.clone(), lifetime, Some(key));

        self
    }

    /// Resolves a dependency using the dependency injection manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use yadir::{deps, let_deps};
    /// use yadir::core::contracts::{DIBuilder};
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
    ///     let mut manager = DIManager::default();
    ///
    ///     manager.build::<Bar>().await;
    ///     manager.build::<Foo>().await;    
    ///
    ///     let foo = manager.resolve::<Foo>().await;
    ///
    ///     assert!(foo.is_some());
    /// }
    /// ```
    pub async fn resolve<T>(&mut self) -> Option<DIObj<T::Output>>
    where
        T: DIBuilder,
    {
        match self.0.get_lifetime::<DIObj<T::Output>>(None) {
            Some(Lifetime::Transient) => self.build::<T>().await,
            Some(Lifetime::Singleton) => {
                let obj = self.0.get::<DIObj<T::Output>>(None).unwrap().extract();
                let sync_obj = DIObj::new(obj);
                Some(sync_obj)
            }
            None => None,
        }
    }

    pub async fn resolve_with_key<T>(&mut self, key: String) -> Option<DIObj<T::Output>>
    where
        T: DIBuilder,
    {
        match self.0.get_lifetime::<DIObj<T::Output>>(Some(key.clone())) {
            Some(Lifetime::Transient) => self.build::<T>().await,
            Some(Lifetime::Singleton) => {
                let obj = self.0.get::<DIObj<T::Output>>(Some(key)).unwrap().extract();
                let sync_obj = DIObj::new(obj);
                Some(sync_obj)
            }
            None => None,
        }
    }

    /// Checks if the dependency injection manager contains a dependency of a given type.
    ///
    /// The `has` method is used to check if the dependency injection manager contains a dependency of a given type.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use yadir::{deps, let_deps};
    /// use yadir::core::contracts::{DIBuilder};
    /// use yadir::core::primitives::{DIObj};
    /// use yadir::core::primitives::DIManager;
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
    ///     let mut manager = DIManager::default();
    ///
    ///     manager.build::<Bar>().await;
    ///     manager.build::<Foo>().await;
    ///
    ///     assert!(manager.has::<DIObj<Bar>>());
    /// }
    /// ```
    pub fn has<T>(&self) -> bool
    where
        T: Any + 'static,
    {
        self.0.has::<T>(None)
    }

    /// Checks if the dependency injection manager contains a dependency of a given type and key.
    ///
    /// The `has_with_key` method is used to check if the dependency injection manager contains a dependency of a given type and key.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_trait::async_trait;
    /// use yadir::{deps, let_deps};
    /// use yadir::core::contracts::{DIBuilder};
    /// use yadir::core::primitives::{DIObj};
    /// use yadir::core::primitives::DIManager;
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
    ///     let mut manager = DIManager::default();
    ///
    ///     manager
    ///         .register::<Bar>(None).await
    ///         .register_with_key::<Foo>(None, String::from("my_key")).await;
    ///
    ///     assert!(manager.has_with_key::<DIObj<Bar>>(String::from("my_key")));
    /// }
    /// ```
    pub fn has_with_key<T>(&self, key: String) -> bool
    where
        T: Any + 'static,
    {
        self.0.has::<T>(Some(key))
    }
}

impl<T, Output> GetInput<Output> for DIObj<T>
where
    T: Clone + 'static,
    Output: GetInputKeys,
{
    fn get_input(manager: &DIManager, key_position: u8) -> Option<Self> {
        let key = Output::get_input_keys()
            .get(key_position as usize)
            .map(|key| key.to_string());
        manager.0.get::<Self>(key).cloned()
    }
}

impl<Output> GetInput<Output> for ()
where
    Output: GetInputKeys,
{
    fn get_input(_: &DIManager, _key_position: u8) -> Option<Self> {
        Some(())
    }
}

impl<S, T, Output> GetInput<Output> for (S, T)
where
    S: GetInput<Output>,
    T: GetInput<Output>,
    Output: GetInputKeys,
{
    fn get_input(manager: &DIManager, key_position: u8) -> Option<Self> {
        S::get_input(manager, key_position)
            .and_then(|s| T::get_input(manager, key_position + 1).map(|t| (s, t)))
    }
}