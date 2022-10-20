use std::path::{Path, PathBuf};

use inflector::Inflector;
use relative_path::{RelativePath, RelativePathBuf};
use swc_atoms::JsWord;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;

use crate::loader::NODE_MODULES_PREFIX;

pub static EXTENSIONS: &'static [&'static str] = &["ts", "tsx", "js", "jsx", "mjs"];

use super::import::ImportTransformer;

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

pub struct Externals {
    root: PathBuf,
}

impl Externals {
    pub fn new(root: PathBuf) -> Externals {
        Externals { root }
    }
    pub fn resolve(&self, path: &RelativePath) -> bool {
        if let Some(ext) = path.extension() {
            if !EXTENSIONS.contains(&ext) {
                return false;
            }
        }

        if path.to_path(&self.root).exists() {
            return true;
        }

        for ext in EXTENSIONS {
            if path.with_extension(*ext).to_path(&self.root).exists() {
                return true;
            }
        }

        false
    }
}

impl ImportTransformer for Externals {
    fn rewrite_import(
        &self,
        _file: &RelativePath,
        mut import: ImportDecl,
        items: &mut Vec<ModuleItem>,
    ) -> Option<ImportDecl> {
        if import.src.value.starts_with(".") || import.src.value.starts_with("/") {
            return Some(import);
        }
        // if !self.resolve(&RelativePathBuf::from(import.src.value.to_string())) {
        //     return Some(import);
        // }

        let node = import.clone();

        let path: JsWord = format!("{}{}", NODE_MODULES_PREFIX, import.src.value).into();
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

        None
    }
}
