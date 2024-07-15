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
yadir = "0.1.0"
```

Create a new registry and register your dependencies, after implementing the `DIBuilder` trait for each one of them:
```rust
use crate::core::{DIBuilder, DIManager, DIObj};
use async_trait::async_trait;
use dyn_clone::{clone_trait_object, DynClone};

// dyn-clone package is used to implement the Clone trait for boxed trait objects
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

#[tokio::main]
async fn main() {
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
```

### **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.