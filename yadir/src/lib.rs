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
    use crate::core::contracts::DIBuilder;
    use crate::core::primitives::{DIManager, DIObj};
    use crate::{deps, let_deps};
    use async_trait::async_trait;
    use dyn_clone::{clone_trait_object, DynClone};

    clone_trait_object!(Printer);
    clone_trait_object!(Writer);

    trait Printer: Sync + Send + DynClone {
        fn print(&self) -> String;
    }

    trait Writer: Sync + Send + DynClone {
        fn write(&self) -> String;
    }

    #[derive(Clone)]
    struct Bar;

    impl Printer for Bar {
        fn print(&self) -> String {
            "bar".to_string()
        }
    }

    #[derive(Clone)]
    struct Baz;

    impl Writer for Baz {
        fn write(&self) -> String {
            "baz".to_string()
        }
    }

    #[derive(Clone)]
    struct Foo {
        printer: Box<dyn Printer>,
        writer: Box<dyn Writer>,
    }

    impl Foo {
        fn new(printer: Box<dyn Printer>, writer: Box<dyn Writer>) -> Self {
            Self { printer, writer }
        }

        fn print(&self) -> String {
            format!("foo {} {}", self.printer.print(), self.writer.write())
        }
    }

    #[async_trait]
    impl DIBuilder for Bar {
        type Input = deps!();
        type Output = Box<dyn Printer>;

        async fn build(_: Self::Input) -> Self::Output {
            Box::new(Self)
        }
    }

    #[async_trait]
    impl DIBuilder for Baz {
        type Input = deps!();
        type Output = Box<dyn Writer>;

        async fn build(_: Self::Input) -> Self::Output {
            Box::new(Self)
        }
    }

    #[async_trait]
    impl DIBuilder for Foo {
        type Input = deps!(Box<dyn Printer>, Box<dyn Writer>);
        type Output = Self;

        async fn build(input: Self::Input) -> Self::Output {
            let_deps!(printer, writer <- input);

            Self::new(printer, writer)
        }
    }

    #[tokio::test]
    async fn test_di_manager() {
        let mut manager = DIManager::default();

        manager.build::<Bar>().await;
        manager.build::<Baz>().await;
        let foo = manager.build::<Foo>().await.unwrap().extract();

        assert_eq!(foo.print(), "foo bar baz");
    }
}
