use std::collections::HashMap;

use inflector::Inflector;
use swc_atoms::JsWord;
use swc_common::{Span, DUMMY_SP};
use swc_ecma_ast::{
    AssignExpr, BindingIdent, CallExpr, ComputedPropName, Decl, Expr, Ident, ImportDecl,
    ImportDefaultSpecifier, ImportNamedSpecifier, ImportSpecifier, Lit, MemberExpr, MemberProp,
    Module, ModuleDecl, ModuleExportName, ModuleItem, Pat, Stmt, VarDecl, VarDeclKind,
    VarDeclarator,
};
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::loader::NODE_MODULES_PREFIX;

#[derive(Debug, Default)]
pub struct ImportTransform(HashMap<JsWord, JsWord>);

impl ImportTransform {
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

macro_rules! var_decl {
    ($name: expr, $init: expr) => {
        VarDecl {
            kind: VarDeclKind::Const,
            span: DUMMY_SP,
            declare: false,
            decls: vec![
                //
                VarDeclarator {
                    span: DUMMY_SP,
                    name: Pat::Ident(BindingIdent {
                        id: $name,
                        type_ann: None,
                    }),
                    definite: false,
                    init: Some($init.into()),
                },
            ],
        }
    };
}

impl ImportTransform {
    fn process_import(&self, mut node: ImportDecl, items: &mut Vec<ModuleItem>) {
        if node.src.value.starts_with("./") || node.src.value.starts_with("/") {
            items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(node)));
            return;
        }

        let mut import = node.clone();

        let path: JsWord = format!("{}{}", NODE_MODULES_PREFIX, node.src.value).into();
        import.src = Box::new(path.into());

        let local = Ident::new(
            format!("$importCJS_${}", node.src.value.to_camel_case()).into(),
            DUMMY_SP,
        );

        import.specifiers = vec![ImportSpecifier::Default(ImportDefaultSpecifier {
            local: local.clone(),
            span: DUMMY_SP,
        })];

        items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import)));

        for specifier in node.specifiers {
            match specifier {
                ImportSpecifier::Default(default) => {
                    let decl = var_decl!(default.local.clone(), Expr::Ident(local.clone()));

                    items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(decl.into()))));
                }
                ImportSpecifier::Named(named) => {
                    let exported = named
                        .imported
                        .map(|m| match m {
                            ModuleExportName::Ident(ident) => MemberProp::Ident(ident),
                            ModuleExportName::Str(str) => MemberProp::Computed(ComputedPropName {
                                span: DUMMY_SP,
                                expr: Expr::Lit(Lit::Str(str)).into(),
                            }),
                        })
                        .unwrap_or_else(|| MemberProp::Ident(named.local.clone()));

                    let expr = MemberExpr {
                        span: DUMMY_SP,
                        obj: Expr::Ident(local.clone()).into(),
                        prop: exported,
                    };

                    let decl = var_decl!(named.local, expr);
                    items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(decl.into()))));
                }
                ImportSpecifier::Namespace(star) => {}
            }
        }
    }

    fn process(&mut self, module_items: &mut Vec<ModuleItem>) {
        let mut updated_items = Vec::with_capacity(module_items.len());

        for item in module_items.drain(..) {
            let import = match item {
                ModuleItem::ModuleDecl(ModuleDecl::Import(import)) => import,
                item => {
                    updated_items.push(item);
                    continue;
                }
            };

            self.process_import(import, &mut updated_items);
        }

        *module_items = updated_items;
    }
}

impl VisitMut for ImportTransform {
    // fn visit_mut_import_decl(&mut self, node: &mut ImportDecl) {
    //     node.visit_mut_children_with(self);

    //     if !node.src.value.starts_with("./") {
    //         let path: JsWord = format!("{}{}", NODE_MODULES_PREFIX, node.src.value).into();
    //         node.src = Box::new(path.into());

    //         // for specifier in node.specifiers {
    //         //     specifier.
    //         // }
    //     }
    // }

    fn visit_mut_module(&mut self, node: &mut Module) {
        node.visit_mut_children_with(self);

        self.process(&mut node.body);

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
