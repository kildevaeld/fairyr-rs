use std::collections::HashMap;

use inflector::Inflector;
use swc_atoms::JsWord;
use swc_common::Span;
use swc_ecma_ast::{
    AssignExpr, CallExpr, Expr, Ident, ImportDecl, ImportDefaultSpecifier, ImportSpecifier, Lit,
    Module, ModuleDecl, ModuleItem, VarDecl,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::loader::NODE_MODULES_PREFIX;

#[derive(Debug, Default)]
pub struct RequireTransform(HashMap<JsWord, JsWord>);

impl RequireTransform {
    pub fn process_call(&mut self, node: &mut CallExpr) -> Option<JsWord> {
        if let Some(expr) = node.callee.as_expr() {
            if let Some(ident) = expr.as_ident() {
                if ident.sym.as_bytes() == b"require" {
                    let first = match node.args.first_mut() {
                        Some(first) => first,
                        None => return None,
                    };

                    if !first.expr.is_lit() {
                        return None;
                    }

                    let lit = first.expr.as_mut_lit().unwrap();

                    let name = match lit {
                        Lit::Str(ident) => ident,
                        _ => return None,
                    };

                    if !name.value.starts_with("./") && !name.value.starts_with("/") {
                        let path: JsWord = format!("{}{}", NODE_MODULES_PREFIX, name.value).into();
                        let name: JsWord =
                            format!("${}_require$", name.value.to_string().to_snake_case()).into();

                        self.0.insert(name.clone(), path);

                        return Some(name);
                    }
                }
            }
        }

        None
    }
}

impl VisitMut for RequireTransform {
    fn visit_mut_module(&mut self, node: &mut Module) {
        node.visit_mut_children_with(self);

        if self.0.is_empty() {
            return;
        }

        let imports = self
            .0
            .iter()
            .map(|(name, import)| {
                //
                let import = ImportDecl {
                    span: Span::default(),
                    asserts: None,
                    type_only: false,
                    src: Box::new(import.clone().into()),
                    specifiers: vec![ImportSpecifier::Default(ImportDefaultSpecifier {
                        span: Span::default(),
                        local: Ident::new(name.clone(), Span::default()),
                    })],
                };

                ModuleItem::ModuleDecl(ModuleDecl::Import(import))
            })
            .collect::<Vec<_>>();

        let old = std::mem::replace(&mut node.body, imports);

        node.body.extend(old);
    }

    fn visit_mut_assign_expr(&mut self, node: &mut AssignExpr) {
        node.visit_mut_children_with(self);

        let name = match node.right.as_mut() {
            Expr::Call(call) => match self.process_call(call) {
                Some(ret) => ret,
                None => return,
            },
            _ => return,
        };

        node.right = Box::new(Expr::Ident(Ident::new(name, Span::default())));
    }

    fn visit_mut_var_decl(&mut self, node: &mut VarDecl) {
        node.visit_mut_children_with(self);

        for n in &mut node.decls {
            let name = if let Some(init) = &mut n.init {
                if let Some(call_expr) = init.as_mut_call() {
                    self.process_call(call_expr)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(name) = name {
                n.init = Some(Box::new(Expr::Ident(Ident::new(name, Span::default()))));
            }
        }
    }
}
