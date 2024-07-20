#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use claim::assert_some;
    use yadir::core::contracts::DIBuilder;
    use yadir::core::primitives::{DIManager, DIObj};
    use yadir::DIBuilder;
    use yadir::{deps, let_deps};

    #[tokio::test]
    async fn test_di_builder_proc_macro_for_all_named_fields_as_deps() {
        #[derive(Clone, DIBuilder)]
        struct Bar;

        #[derive(Clone, DIBuilder)]
        struct Baz;

        #[derive(Clone, DIBuilder)]
        struct Foo {
            #[deps]
            bar: Bar,
            #[deps]
            baz: Baz,
        }

        let mut manager = DIManager::default();
        manager.build::<Bar>().await;
        manager.build::<Baz>().await;
        let foo = manager.build::<Foo>().await;

        assert_some!(foo);
    }

    #[tokio::test]
    async fn test_di_builder_proc_macro_for_all_unnamed_fields_as_deps() {
        #[derive(Clone, DIBuilder)]
        struct Bar;

        #[derive(Clone, DIBuilder)]
        struct Baz;

        #[derive(Clone, DIBuilder)]
        struct Foo(#[deps] Bar, #[deps] Baz);

        let mut manager = DIManager::default();
        manager.build::<Bar>().await;
        manager.build::<Baz>().await;
        let foo = manager.build::<Foo>().await;

        assert_some!(foo);
    }

    #[tokio::test]
    async fn test_di_builder_proc_macro_for_all_named_fields_as_deps_with_new_build_method() {
        #[derive(Clone, DIBuilder)]
        struct Bar;

        #[derive(Clone, DIBuilder)]
        struct Baz;

        #[derive(Clone, DIBuilder)]
        #[build_method("new")]
        struct Foo {
            #[deps]
            bar: Bar,
            #[deps]
            baz: Baz,
        }

        impl Foo {
            fn new(bar: Bar, baz: Baz) -> Self {
                Self { bar, baz }
            }
        }

        let mut manager = DIManager::default();
        manager.build::<Bar>().await;
        manager.build::<Baz>().await;
        let foo = manager.build::<Foo>().await;

        assert_some!(foo);
    }

    #[tokio::test]
    async fn test_di_builder_proc_macro_for_all_unnamed_fields_as_deps_with_new_build_method() {
        #[derive(Clone, DIBuilder)]
        struct Bar;

        #[derive(Clone, DIBuilder)]
        struct Baz;

        #[derive(Clone, DIBuilder)]
        #[build_method("new")]
        struct Foo(#[deps] Bar, #[deps] Baz);

        impl Foo {
            fn new(bar: Bar, baz: Baz) -> Self {
                Self(bar, baz)
            }
        }

        let mut manager = DIManager::default();
        manager.build::<Bar>().await;
        manager.build::<Baz>().await;
        let foo = manager.build::<Foo>().await;

        assert_some!(foo);
    }

    #[tokio::test]
    async fn test_di_builder_proc_macro_for_all_named_fields_as_deps_with_default_build_method() {
        #[derive(Default, Clone, DIBuilder)]
        struct Bar;

        #[derive(Default, Clone, DIBuilder)]
        struct Baz;

        #[derive(Default, Clone, DIBuilder)]
        #[build_method("default")]
        struct Foo {
            #[deps]
            bar: Bar,
            #[deps]
            baz: Baz,
        }

        let mut manager = DIManager::default();
        manager.build::<Bar>().await;
        manager.build::<Baz>().await;
        let foo = manager.build::<Foo>().await;

        assert_some!(foo);
    }

    #[tokio::test]
    async fn test_di_builder_proc_macro_for_all_unnamed_fields_as_deps_with_default_build_method() {
        #[derive(Default, Clone, DIBuilder)]
        struct Bar;

        #[derive(Default, Clone, DIBuilder)]
        struct Baz;

        #[derive(Default, Clone, DIBuilder)]
        #[build_method("default")]
        struct Foo(#[deps] Bar, #[deps] Baz);

        let mut manager = DIManager::default();
        manager.build::<Bar>().await;
        manager.build::<Baz>().await;
        let foo = manager.build::<Foo>().await;

        assert_some!(foo);
    }
}
