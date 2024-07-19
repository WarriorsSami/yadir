#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use claim::assert_some;
    use yadir::core::contracts::DIBuilder;
    use yadir::core::primitives::{DIManager, DIObj};
    use yadir::DIBuilder;
    use yadir::{deps, let_deps};

    #[tokio::test]
    async fn test_di_builder_proc_macro_all_fields_as_deps() {
        #[derive(Clone, DIBuilder)]
        struct Bar;

        #[derive(Clone, DIBuilder)]
        struct Baz;

        #[derive(Clone, DIBuilder)]
        struct Foo {
            #[di_build]
            bar: Bar,
            #[di_build]
            baz: Baz,
        }
        
        let mut manager = DIManager::default();
        manager.build::<Bar>().await;
        manager.build::<Baz>().await;
        let foo = manager.build::<Foo>().await;

        assert_some!(foo);
    }
}
