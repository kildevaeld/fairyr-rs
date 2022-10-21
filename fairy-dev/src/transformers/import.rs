use std::collections::HashSet;

use relative_path::RelativePath;
use swc_common::sync::Lrc;
use swc_ecma_ast::{ImportDecl, Module, ModuleDecl, ModuleItem};

pub trait ImportTransformer {
    fn rewrite_import(
        &self,
        file: &RelativePath,
        import: ImportDecl,
        items: &mut Vec<ModuleItem>,
    ) -> Option<ImportDecl>;
}

impl ImportTransformer for Box<dyn ImportTransformer + Send + Sync> {
    fn rewrite_import(
        &self,
        file: &RelativePath,
        import: ImportDecl,
        items: &mut Vec<ModuleItem>,
    ) -> Option<ImportDecl> {
        (&**self).rewrite_import(file, import, items)
    }
}

#[derive(Default, Clone)]
pub struct ImportTransform {
    plugins: Lrc<Vec<Box<dyn ImportTransformer + Send + Sync>>>,
}

impl ImportTransform {
    pub fn new(plugins: Lrc<Vec<Box<dyn ImportTransformer + Send + Sync>>>) -> ImportTransform {
        ImportTransform { plugins }
    }
}

impl ImportTransform {
    fn process_import(
        &self,
        file: &RelativePath,
        mut node: ImportDecl,
        items: &mut Vec<ModuleItem>,
    ) {
        for plugin in self.plugins.iter() {
            if let Some(next) = plugin.rewrite_import(file, node, items) {
                node = next
            } else {
                return;
            }
        }

        items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(node)));
    }

    fn process(&self, file: &RelativePath, module_items: &mut Vec<ModuleItem>) {
        let mut updated_items = Vec::with_capacity(module_items.len());

        for item in module_items.drain(..) {
            let import = match item {
                ModuleItem::ModuleDecl(ModuleDecl::Import(import)) => import,
                item => {
                    updated_items.push(item);
                    continue;
                }
            };

            self.process_import(file, import, &mut updated_items);
        }

        // TODO: Rewrite this
        let mut seen = HashSet::<_>::default();
        *module_items = updated_items
            .into_iter()
            .filter(move |item| {
                //
                match item {
                    ModuleItem::ModuleDecl(ModuleDecl::Import(import)) => {
                        if seen.contains(&import.specifiers) {
                            false
                        } else {
                            seen.insert(import.specifiers.clone());
                            true
                        }
                    }
                    _ => true,
                }
            })
            .collect();
    }

    pub fn process_module(&self, path: &RelativePath, module: &mut Module) {
        self.process(path, &mut module.body)
    }
}

// impl VisitMut for ImportTransform {
//     fn visit_mut_module(&mut self, node: &mut Module) {
//         node.visit_mut_children_with(self);

//         self.process(&mut node.body);
//     }
// }
