use std::collections::HashSet;

use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};

#[derive(Default)]
pub struct ImportResolver(pub HashSet<String>);

impl VisitMut for ImportResolver {
    fn visit_mut_import_decl(&mut self, import: &mut ImportDecl) {
        import.visit_mut_children_with(self);
        self.0.insert(import.src.value.to_string());
    }
}

#[derive(Default)]
pub struct ImportResolver2;

impl VisitMut for ImportResolver2 {
    fn visit_mut_module(&mut self, module: &mut Module) {
        module.visit_mut_children_with(self);

        module.body = module
            .body
            .drain(..)
            .filter(|item| {
                //
                match item {
                    ModuleItem::ModuleDecl(ModuleDecl::Import(_)) => false,
                    _ => true,
                }
            })
            .collect();
    }
}
