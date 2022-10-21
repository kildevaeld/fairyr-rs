use std::collections::HashMap;

use crate::{compiler::Compiler, loader::NODE_MODULES_PREFIX, transformers::RequireTransform};
use fairy_core::{ModuleType, Package};
use relative_path::RelativePathBuf;
use swc_atoms::js_word;
use swc_bundler::{Bundle as SWCBundle, ModuleRecord};
use swc_common::{sync::Lrc, FileName, SourceMap, Span};
use swc_ecma_ast::{
    Bool, Expr, Ident, KeyValueProp, Lit, MemberExpr, MemberProp, MetaPropExpr, MetaPropKind,
    PropName, Str,
};
use swc_ecma_codegen::{
    text_writer::{omit_trailing_semi, JsWriter, WriteJs},
    Emitter,
};
use swc_ecma_visit::VisitMut;

pub struct Bundle {
    cm: Lrc<SourceMap>,
    bundle: SWCBundle,
}

impl Bundle {
    pub fn to_bytes(&self, minify: bool) -> anyhow::Result<Vec<u8>> {
        let mut buf = vec![];

        {
            let wr = JsWriter::new(self.cm.clone(), "\n", &mut buf, None);
            let mut emitter = Emitter {
                cfg: swc_ecma_codegen::Config {
                    minify,
                    ..Default::default()
                },
                cm: self.cm.clone(),
                comments: None,
                wr: if minify {
                    Box::new(omit_trailing_semi(wr)) as Box<dyn WriteJs>
                } else {
                    Box::new(wr) as Box<dyn WriteJs>
                },
            };

            emitter.emit_module(&self.bundle.module)?;
        }

        Ok(buf)
    }

    pub fn to_string(&self, minify: bool) -> anyhow::Result<String> {
        let bytes = self.to_bytes(minify)?;
        let string = String::from_utf8(bytes)?;
        Ok(string)
    }
}

#[derive(Debug)]
pub struct Bundler {
    minify: bool,
    inline: bool, // resolver: NodeResolver,
}

impl Default for Bundler {
    fn default() -> Self {
        Bundler {
            minify: false,
            inline: true,
        }
    }
}

impl Bundler {
    pub fn bundle(&self, compiler: &Compiler, name: &str) -> anyhow::Result<Bundle> {
        let package = compiler.resolve(name)?;
        self.bundle_package(compiler, package)
    }

    pub fn bundle_package(&self, compiler: &Compiler, package: Package) -> anyhow::Result<Bundle> {
        let mut externals = Vec::default();

        for (name, _) in package.pkgjson.peer_dependencies.into_iter() {
            externals.push(name.into());
        }

        let mut bundler = compiler.create_bundler(swc_bundler::Config {
            require: true,
            disable_inliner: !self.inline,
            external_modules: externals,
            disable_fixer: self.minify,
            disable_hygiene: self.minify,
            disable_dce: false,
            module: swc_bundler::ModuleType::Es,
        });

        let mut entries = HashMap::new();

        let dep = RelativePathBuf::from(format!(
            "{}{}/{}",
            NODE_MODULES_PREFIX, &package.pkgjson.name, package.entry.path
        ));

        let resolved = FileName::Real(dep.to_path(compiler.root()));

        entries.insert(package.pkgjson.name.clone(), resolved);

        let mut bundles = compiler.run_handler(|| bundler.bundle(entries))?;

        let mut bundle = bundles.pop().unwrap();

        if package.entry.kind == ModuleType::Commonjs {
            let mut visitor = RequireTransform::default();
            visitor.visit_mut_module(&mut bundle.module);
        } else {
            compiler
                .transformer
                .process_module(&package.entry.path, &mut bundle.module);
        }

        Ok(Bundle {
            bundle,
            cm: compiler.cm().clone(),
        })
    }

    // pub fn bundle_dependency(
    //     &self,
    //     compiler: &Compiler,
    //     dependency: &Dependency,
    // ) -> anyhow::Result<Bundle> {
    //     self.bundle_package(compiler, dependency.package()?)
    // }
}

struct Hook;

impl swc_bundler::Hook for Hook {
    fn get_import_meta_props(
        &self,
        span: Span,
        module_record: &ModuleRecord,
    ) -> Result<Vec<KeyValueProp>, anyhow::Error> {
        let file_name = module_record.file_name.to_string();

        Ok(vec![
            KeyValueProp {
                key: PropName::Ident(Ident::new(js_word!("url"), span)),
                value: Box::new(Expr::Lit(Lit::Str(Str {
                    span,
                    raw: None,
                    value: file_name.into(),
                }))),
            },
            KeyValueProp {
                key: PropName::Ident(Ident::new(js_word!("main"), span)),
                value: Box::new(if module_record.is_entry {
                    Expr::Member(MemberExpr {
                        span,
                        obj: Box::new(Expr::MetaProp(MetaPropExpr {
                            span,
                            kind: MetaPropKind::ImportMeta,
                        })),
                        prop: MemberProp::Ident(Ident::new(js_word!("main"), span)),
                    })
                } else {
                    Expr::Lit(Lit::Bool(Bool { span, value: false }))
                }),
            },
        ])
    }
}
