//! # yadir
//!
//! `yadir` is yet another simple dependency injection registry for Rust.

pub mod core;
pub mod macros;

#[cfg(feature = "derive")]
extern crate yadir_derive;

#[cfg(feature = "derive")]
pub use yadir_derive::DIBuilder;

#[cfg(test)]
mod tests {
    use crate::core::contracts::{DIBuilder, GetInputKeys};
    use crate::core::primitives::{DIManager, DIObj, Lifetime};
    use crate::{deps, let_deps};
    use async_trait::async_trait;
    use claim::assert_some;
    use dyn_clone::{clone_trait_object, DynClone};
    use uuid::Uuid;
    use yadir_derive::DIBuilder;

    clone_trait_object!(Printer);
    clone_trait_object!(Writer);

    trait Printer: Sync + Send + DynClone {
        fn print(&self) -> String;
    }

    trait Writer: Sync + Send + DynClone {
        fn write(&self) -> String;
    }

    #[derive(Clone, DIBuilder)]
    #[build_as(Box<dyn Printer>)]
    struct Bar;

    impl Printer for Bar {
        fn print(&self) -> String {
            "bar".to_string()
        }
    }
    
    #[derive(Clone, DIBuilder)]
    #[build_as(Box<dyn Writer>)]
    struct Baz;

    impl Writer for Baz {
        fn write(&self) -> String {
            "baz".to_string()
        }
    }

    #[derive(Clone, DIBuilder)]
    #[build_method("new")]
    struct Foo {
        id: Uuid,
        #[deps()]
        printer: Box<dyn Printer>,
        #[deps()]
        writer: Box<dyn Writer>,
    }

    impl Foo {
        fn new(printer: Box<dyn Printer>, writer: Box<dyn Writer>) -> Self {
            let id = Uuid::new_v4();
            Self {
                id,
                printer,
                writer,
            }
        }

        fn print(&self) -> String {
            format!("foo {} {}", self.printer.print(), self.writer.write())
        }

        fn id(&self) -> Uuid {
            self.id
        }
    }

    #[tokio::test]
    async fn test_di_manager_for_deps_transient_lifetimes() {
        let mut manager = DIManager::default();

        manager
            .register::<Bar>(Some(Lifetime::Transient))
            .await
            .register::<Baz>(Some(Lifetime::Transient))
            .await
            .register::<Foo>(Some(Lifetime::Transient))
            .await;

        let foo1 = manager.resolve::<Foo>().await;
        assert_some!(foo1.clone());

        let foo1 = foo1.unwrap().extract();
        assert_eq!(foo1.print(), "foo bar baz");

        let foo2 = manager.resolve::<Foo>().await;
        assert_some!(foo2.clone());

        let foo2 = foo2.unwrap().extract();
        assert_eq!(foo2.print(), "foo bar baz");

        assert_ne!(foo1.id(), foo2.id());
    }

    #[tokio::test]
    async fn test_di_manager_for_deps_singleton_lifetimes() {
        let mut manager = DIManager::default();

        manager
            .register::<Bar>(Some(Lifetime::Transient))
            .await
            .register::<Baz>(Some(Lifetime::Transient))
            .await
            .register::<Foo>(Some(Lifetime::Singleton))
            .await;

        let foo1 = manager.resolve::<Foo>().await;
        assert_some!(foo1.clone());

        let foo1 = foo1.unwrap().extract();
        assert_eq!(foo1.print(), "foo bar baz");

        let foo2 = manager.resolve::<Foo>().await;
        assert_some!(foo2.clone());

        let foo2 = foo2.unwrap().extract();
        assert_eq!(foo2.print(), "foo bar baz");

        assert_eq!(foo1.id(), foo2.id());
    }

    #[tokio::test]
    async fn test_di_manager_for_not_resolving_unregistered_deps() {
        let mut manager = DIManager::default();

        manager
            .register::<Baz>(Some(Lifetime::Transient))
            .await
            .register::<Bar>(Some(Lifetime::Transient))
            .await;

        let foo = manager.resolve::<Foo>().await;
        assert!(foo.is_none());
    }

    #[tokio::test]
    async fn test_di_manager_for_keyed_dependencies() {
        #[derive(Clone, DIBuilder)]
        struct FooBar {
            #[deps(key("my_foo"))]
            foo: Foo,
            #[deps()]
            printer: Box<dyn Printer>,
        }

        let mut manager = DIManager::default();

        manager
            .register::<Bar>(Some(Lifetime::Transient))
            .await
            .register::<Baz>(Some(Lifetime::Transient))
            .await
            .register::<Foo>(Some(Lifetime::Singleton))
            .await
            .register_with_key::<Foo>(Some(Lifetime::Singleton), String::from("my_foo"))
            .await
            .register::<FooBar>(Some(Lifetime::Transient))
            .await;

        let foo_bar = manager.resolve::<FooBar>().await;
        assert_some!(foo_bar.clone());

        let foo_bar = foo_bar.unwrap().extract();
        assert_eq!(foo_bar.foo.print(), "foo bar baz");

        let foo_with_key = manager
            .resolve_with_key::<Foo>(String::from("my_foo"))
            .await;
        assert_some!(foo_with_key.clone());

        let foo_with_key = foo_with_key.unwrap().extract();
        assert_eq!(foo_with_key.print(), "foo bar baz");

        let foo = manager.resolve::<Foo>().await;
        assert_some!(foo.clone());

        let foo = foo.unwrap().extract();
        assert_eq!(foo.print(), "foo bar baz");

        assert_ne!(foo_with_key.id(), foo.id());
    }
}
