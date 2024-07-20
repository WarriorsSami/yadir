# Yadir

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/WarriorsSami/yadir/rust.yml)
[![Crates.io Version](https://img.shields.io/crates/v/yadir)](https://crates.io/crates/yadir)

### **Yadir is yet another dependency injection registry for Rust.**

---

### **About**

Yadir's API is heavily inspired by the [Microsoft.Extensions.DependencyInjection](https://learn.microsoft.com/en-us/dotnet/core/extensions/dependency-injection) library for .NET. It provides a simple and easy way to register and resolve dependencies in your Rust application at runtime.

Its initial implementation is based on the [Registry design pattern](https://willcrichton.net/rust-api-type-patterns/registries.html), formulated by Will Crichton in his book about [Type-Driven API Design in Rust](https://willcrichton.net/rust-api-type-patterns/introduction.html).

### **Usage**

Add Yadir to your `Cargo.toml` file:
```toml
[dependencies]
yadir = { version = "0.2.3", features = ["derive"] }
```

Create a new registry and register your dependencies, after implementing the `DIBuilder` trait for each one of them:
```rust
use yadir::core::contracts::DIBuilder;
use yadir::core::primitives::{DIManager, DIObj};
use yadir::{deps, let_deps};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};
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
    #[deps]
    printer: Box<dyn Printer>,
    #[deps]
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

#[tokio::main]
async fn main() {
    let mut manager = DIManager::default();
    manager.build::<Bar>().await;
    manager.build::<Baz>().await;
    
    let foo = manager.build::<Foo>().await.unwrap().extract();
    
    assert_eq!(foo.print(), "foo bar baz");
}
```

### **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.