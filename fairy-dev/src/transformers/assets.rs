use relative_path::RelativePath;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;

use super::ImportTransformer;

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

pub struct AssetsTransform {}

impl AssetsTransform {
    pub fn new() -> AssetsTransform {
        AssetsTransform {}
    }
}

static ASSETS_EXTENSIONS: &'static [&'static str] = &["jpg", "jpeg", "png", "gif", "webp", "ico"];

impl ImportTransformer for AssetsTransform {
    fn rewrite_import(
        &self,
        file: &RelativePath,

        import: ImportDecl,
        items: &mut Vec<ModuleItem>,
    ) -> Option<ImportDecl> {
        if !import.src.value.starts_with(".") && !import.src.value.starts_with("/") {
            return Some(import);
        }

        if import.specifiers.len() > 1 {
            return Some(import);
        }

        let parent = file.parent().unwrap_or_else(|| &RelativePath::new("/"));

        let src = parent.join_normalized(import.src.value.to_string());

        match src.extension() {
            Some(ext) => {
                if !ASSETS_EXTENSIONS.contains(&ext) {
                    return Some(import);
                }
            }
            None => return Some(import),
        };

        let src = format!("/{}", src);

        let first = import.specifiers.get(0).expect("import specifier");

        if let ImportSpecifier::Default(default) = first {
            log::debug!("asset {}", default.local);
            let var = var_decl!(
                default.local.clone(),
                Expr::Lit(Lit::Str(src.to_string().into()))
            );

            items.push(ModuleItem::Stmt(Stmt::Decl(Decl::Var(var.into()))));
        } else {
            return Some(import);
        }

        None
    }
}
