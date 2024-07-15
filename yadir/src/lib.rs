//! # yadir
//!
//! `yadir` is yet another simple dependency injection registry for Rust.

pub mod core {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    /// A simple type map that stores values by their type.
    #[derive(Default)]
    pub struct TypeMap(HashMap<TypeId, Box<dyn Any>>);

    impl TypeMap {
        /// Inserts a value into the map with its inferred type as the key.
        ///
        /// # Examples
        ///
        /// ```
        /// use yadir::core::TypeMap;
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
        /// use yadir::core::TypeMap;
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
        /// use yadir::core::TypeMap;
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
        /// use yadir::core::TypeMap;
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
        /// As ['GetInput'](GetInput) is using a recursive implementation via inductive traits, you can use tuples to pass multiple dependencies.
        /// Also, you should wrap each dependency in a [`DIObj`](DIObj) to ensure thread safety and that the dependency can be easily
        /// resolved by the dependency injection manager.
        /// 
        /// # Examples
        /// 
        /// ```
        /// use async_trait::async_trait;
        /// use yadir::core::{DIBuilder, DIManager, DIObj, GetInput};
        ///
        /// #[derive(Clone)]
        /// struct Bar;
        /// 
        /// #[derive(Clone)]
        /// struct Foo(Bar);
        /// 
        /// #[async_trait]
        /// impl DIBuilder for Bar {
        ///    type Input = ();
        ///    type Output = Self;
        /// 
        ///     async fn build(_: Self::Input) -> Self::Output {
        ///         Self
        ///     }
        /// }
        /// 
        /// #[async_trait]
        /// impl DIBuilder for Foo {
        ///     type Input = (DIObj<Bar>, ());
        ///     type Output = Self;
        ///
        ///     async fn build((bar, _): Self::Input) -> Self::Output {
        ///         Self(bar.lock().unwrap().clone())
        ///     }
        /// }
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
        /// use yadir::core::{DIBuilder, DIManager, DIObj, GetInput};
        ///
        /// #[derive(Clone)]
        /// struct Bar;
        /// 
        /// #[derive(Clone)]
        /// struct Foo(Bar);
        /// 
        /// #[async_trait]
        /// impl DIBuilder for Bar {
        ///    type Input = ();
        ///    type Output = Self;
        /// 
        ///     async fn build(_: Self::Input) -> Self::Output {
        ///         Self
        ///     }
        /// }
        /// 
        /// #[async_trait]
        /// impl DIBuilder for Foo {
        ///     type Input = (DIObj<Bar>, ());
        ///     type Output = Self;
        ///
        ///     async fn build((bar, _): Self::Input) -> Self::Output {
        ///         Self(bar.lock().unwrap().clone())
        ///     }
        /// }
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
        async fn build(input: Self::Input) -> Self::Output;
    }

    /// A wrapper type for a thread-safe reference counted mutex to handle thread-safe sharing of embedded dependencies.
    pub type DIObj<T> = Arc<Mutex<T>>;

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
        /// # use yadir::core::{DIBuilder, DIObj, GetInput};
        /// use yadir::core::DIManager;
        ///
        /// # #[derive(Clone)]
        /// # struct Bar;
        /// # 
        /// # #[derive(Clone)]
        /// # struct Foo(Bar);
        /// # 
        /// # #[async_trait]
        /// # impl DIBuilder for Bar {
        /// #    type Input = ();
        /// #    type Output = Self;
        /// # 
        /// #     async fn build(_: Self::Input) -> Self::Output {
        /// #         Self
        /// #     }
        /// # }
        /// # 
        /// # #[async_trait]
        /// # impl DIBuilder for Foo {
        /// #     type Input = (DIObj<Bar>, ());
        /// #     type Output = Self;
        /// #
        /// #     async fn build((bar, _): Self::Input) -> Self::Output {
        /// #         Self(bar.lock().unwrap().clone())
        /// #     }
        /// # }
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
            let sync_obj = DIObj::new(Mutex::new(obj));
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
        /// use async_trait::async_trait;
        /// use yadir::core::{DIBuilder, DIObj, GetInput};
        /// use yadir::core::DIManager;
        ///
        /// #[derive(Clone)]
        /// struct Bar;
        ///
        /// # #[derive(Clone)]
        /// # struct Foo(Bar);
        /// # 
        /// #[async_trait]
        /// impl DIBuilder for Bar {
        ///     type Input = ();
        ///     type Output = Self;
        /// 
        ///      async fn build(_: Self::Input) -> Self::Output {
        ///          Self
        ///      }
        /// }
        ///  
        /// # #[async_trait]
        /// # impl DIBuilder for Foo {
        /// #     type Input = (DIObj<Bar>, ());
        /// #     type Output = Self;
        /// #
        /// #     async fn build((bar, _): Self::Input) -> Self::Output {
        /// #         Self(bar.lock().unwrap().clone())
        /// #     }
        /// # }
        /// # 
        /// #[tokio::main]
        /// async fn main() {
        ///     let mut manager = DIManager::default();
        ///
        ///     manager.build::<Bar>().await;
        /// #    manager.build::<Foo>().await; 
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

    /// A trait used to retrieve dependencies from the dependency injection manager.
    /// 
    /// The `GetInput` trait is used to inductively resolve all the dependencies needed to build the implementer type from the dependency injection manager.
    /// It is implemented for the following cases:
    /// - `DIObj<T>`: to retrieve a dependency wrapped in a thread-safe reference counted mutex from the dependency injection manager (**base case**).
    /// - `()`: to return the unit type when no dependencies are needed (**base case**).
    /// - `(S, T)`: to retrieve multiple dependencies by recursively resolving each dependency (**inductive case**).
    pub trait GetInput: Sized {
        fn get_input(manager: &DIManager) -> Option<Self>;
    }

    impl<T: 'static> GetInput for DIObj<T> {
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
}

#[cfg(test)]
mod tests {
    use crate::core::{DIBuilder, DIManager, DIObj};
    use async_trait::async_trait;
    use dyn_clone::{clone_trait_object, DynClone};

    clone_trait_object!(Printer);

    trait Printer: Send + Sync + DynClone {
        fn print(&self) -> String;
    }

    #[derive(Clone)]
    struct Bar;

    impl Printer for Bar {
        fn print(&self) -> String {
            "bar".to_string()
        }
    }

    #[derive(Clone)]
    struct Foo {
        printer: Box<dyn Printer>,
    }

    impl Foo {
        fn new(printer: Box<dyn Printer>) -> Self {
            Self { printer }
        }

        fn print(&self) -> String {
            format!("foo {}", self.printer.print())
        }
    }

    #[async_trait]
    impl DIBuilder for Bar {
        type Input = ();
        type Output = Box<dyn Printer>;

        async fn build(_: Self::Input) -> Self::Output {
            Box::new(Self)
        }
    }

    #[async_trait]
    impl DIBuilder for Foo {
        type Input = (DIObj<Box<dyn Printer>>, ());
        type Output = Self;

        async fn build((printer, _): Self::Input) -> Self::Output {
            Self::new(printer.lock().unwrap().clone())
        }
    }

    #[tokio::test]
    async fn test_di_manager() {
        let mut manager = DIManager::default();

        manager.build::<Bar>().await;
        let foo = manager
            .build::<Foo>()
            .await
            .unwrap()
            .lock()
            .unwrap()
            .clone();

        assert_eq!(foo.print(), "foo bar");
    }
}
