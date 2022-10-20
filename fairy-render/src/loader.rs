use deno_core::{FsModuleLoader, ModuleLoader, ModuleSpecifier};

struct Loader(FsModuleLoader);

impl Loader {
    pub fn new() -> Loader {
        Loader(FsModuleLoader)
    }
}

impl ModuleLoader for Loader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        is_main: bool,
    ) -> Result<ModuleSpecifier, anyhow::Error> {
        self.0.resolve(specifier, referrer, is_main)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        maybe_referrer: Option<ModuleSpecifier>,
        is_dyn_import: bool,
    ) -> std::pin::Pin<Box<deno_core::ModuleSourceFuture>> {
        self.0.load(module_specifier, maybe_referrer, is_dyn_import)
    }

    fn prepare_load(
        &self,
        op_state: Rc<std::cell::RefCell<deno_core::OpState>>,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<String>,
        _is_dyn_import: bool,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), anyhow::Error>>>> {
        op_state
            .borrow_mut()
            .borrow_mut::<Modules>()
            .0
            .push(module_specifier.to_string());
        async { Ok(()) }.boxed_local()
    }
}
