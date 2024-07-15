pub mod core {
    use std::any::{Any, TypeId};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    #[derive(Default)]
    struct TypeMap(HashMap<TypeId, Box<dyn Any>>);

    impl TypeMap {
        pub fn set<T>(&mut self, t: T)
        where
            T: Any + 'static,
        {
            self.0.insert(TypeId::of::<T>(), Box::new(t));
        }

        pub fn get<T>(&self) -> Option<&T>
        where
            T: Any + 'static,
        {
            self.0
                .get(&TypeId::of::<T>())
                .map(|boxed| boxed.downcast_ref::<T>().unwrap())
        }

        pub fn get_mut<T>(&mut self) -> Option<&mut T>
        where
            T: Any + 'static,
        {
            self.0
                .get_mut(&TypeId::of::<T>())
                .map(|boxed| boxed.downcast_mut::<T>().unwrap())
        }

        pub fn has<T>(&self) -> bool
        where
            T: Any + 'static,
        {
            self.0.contains_key(&TypeId::of::<T>())
        }
    }

    #[async_trait]
    pub trait DIBuilder {
        type Input: GetInput;
        type Output: 'static;

        async fn build(input: Self::Input) -> Self::Output;
    }

    pub type DIObj<T> = Arc<Mutex<T>>;

    #[derive(Default)]
    pub struct DIManager(TypeMap);

    impl DIManager {
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
    }

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
