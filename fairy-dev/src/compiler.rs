use anyhow::bail;
use fairy_core::Package;
use pathdiff::diff_paths;
use relative_path::RelativePath;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use swc::{
    config::{JscConfig, Options},
    TransformOutput,
};
use swc_atoms::{js_word, JsWord};
use swc_bundler::{Bundler, ModuleRecord};
use swc_common::{
    collections::AHashMap,
    errors::{ColorConfig, Handler, HANDLER},
    source_map::SourceMap,
    sync::Lrc,
    FilePathMapping, Globals, Span, GLOBALS,
};
use swc_ecma_ast::{
    Bool, EsVersion, Expr, Ident, KeyValueProp, Lit, MemberExpr, MemberProp, MetaPropExpr,
    MetaPropKind, Program, PropName, Str,
};

use swc_ecma_parser::{parse_file_as_module, Syntax, TsConfig};
use swc_ecma_transforms_optimization::inline_globals;

use crate::{
    loader::{FileLoader, Loader},
    resolver::Resolver,
    transformers::{
        AssetsTransform, Externals as ExternalTransform, ImportTransform, ImportTransformer,
    },
};

pub struct Compiler {
    cm: Lrc<SourceMap>,
    root: PathBuf,
    compiler: swc::Compiler,
    handler: Lrc<Handler>,
    globals: Globals,
    resolver: Lrc<Resolver>,
    env: Lrc<AHashMap<JsWord, Expr>>,
    pub(crate) transformer: ImportTransform,
}

impl Compiler {
    pub fn new(root: PathBuf, env: Lrc<AHashMap<JsWord, Expr>>) -> Compiler {
        let file_loader = FileLoader::new(root.clone());

        let cm = Lrc::new(SourceMap::with_file_loader(
            Box::new(file_loader),
            FilePathMapping::empty(),
        ));

        let compiler = swc::Compiler::new(cm.clone());
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, None);
        let globals = Globals::default();

        let resolver = Resolver::new(root.clone());

        let plugins = vec![
            Box::new(ExternalTransform::new()) as Box<dyn ImportTransformer + Send + Sync>,
            Box::new(AssetsTransform::new()),
        ];

        let transformer = ImportTransform::new(Lrc::new(plugins));

        Compiler {
            root,
            cm,
            compiler,
            handler: Lrc::new(handler),
            globals,
            resolver: Arc::new(resolver),
            env,
            transformer,
        }
    }

    pub fn cm(&self) -> &Lrc<SourceMap> {
        &self.cm
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn resolve(&self, name: &str) -> anyhow::Result<Package> {
        match self.resolver.resolve_external(name) {
            Some(ret) => Ok(ret),
            None => bail!("could not resolve"),
        }
    }

    pub fn create_bundler<'a>(
        &'a self,
        config: swc_bundler::Config,
    ) -> swc_bundler::Bundler<'a, Loader, Lrc<Resolver>> {
        let loader = Loader::new(self.cm.clone(), self.env.clone(), self.handler.clone());

        let bundler = Bundler::new(
            &self.globals,
            self.cm.clone(),
            loader,
            self.resolver.clone(),
            config,
            Box::new(Hook),
        );

        bundler
    }

    pub fn compile(&self, path: impl AsRef<Path>) -> anyhow::Result<TransformOutput> {
        let file = self.cm.load_file(path.as_ref())?;

        let syntax = Syntax::Typescript(TsConfig {
            tsx: true,
            ..Default::default()
        });

        let es_version = EsVersion::Es2019;

        let mut module = parse_file_as_module(&file, syntax, es_version, None, &mut Vec::default())
            .expect("parse");

        let rel_path = diff_paths(path.as_ref(), &self.root).expect("relative path");
        let rel_path = RelativePath::from_path(&rel_path).expect("rel path");

        self.transformer
            .clone()
            .process_module(rel_path, &mut module);

        let out = self.run(|| {
            let program = self.compiler.transform(
                &self.handler,
                Program::Module(module),
                false,
                inline_globals(self.env.clone(), Default::default(), Default::default()),
            );

            let out = self.compiler.process_js(
                &self.handler,
                program,
                &Options {
                    config: swc::config::Config {
                        jsc: JscConfig {
                            target: es_version.into(),
                            external_helpers: false.into(),
                            syntax: Some(syntax),

                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )?;

            anyhow::Ok(out)
        })?;

        Ok(out)
    }

    pub fn run<F, R>(&self, func: F) -> R
    where
        F: FnOnce() -> R,
    {
        GLOBALS.set(&self.globals, func)
    }

    pub fn run_handler<F, R>(&self, func: F) -> R
    where
        F: FnOnce() -> R,
    {
        HANDLER.set(&self.handler, func)
    }
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
