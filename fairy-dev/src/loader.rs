use std::path::PathBuf;

use anyhow::Error;
use relative_path::RelativePathBuf;
use swc_atoms::JsWord;
use swc_bundler::{Load, ModuleData};
use swc_common::{
    collections::AHashMap,
    errors::{ColorConfig, Handler},
    pass::Repeated,
    source_map::FileLoader as SwcFileLoader,
    sync::Lrc,
    FileName, Mark, SourceMap,
};
use swc_ecma_ast::{EsVersion, Expr};
use swc_ecma_parser::{parse_file_as_module, EsConfig, Syntax};
use swc_ecma_transforms_base::helpers::{self, Helpers};
use swc_ecma_transforms_optimization::{
    inline_globals, simplifier, simplify::Config as SimplyConfig,
};
use swc_ecma_visit::Fold;

pub static NODE_MODULES_PREFIX: &'static str = "/node_modules/.fairy/";

#[derive(Clone)]
pub struct Loader {
    pub cm: Lrc<SourceMap>,
    pub env: Lrc<AHashMap<JsWord, Expr>>,
    pub globals: Lrc<AHashMap<JsWord, Expr>>,
}

impl Loader {
    pub fn new(cm: Lrc<SourceMap>, env: Lrc<AHashMap<JsWord, Expr>>) -> Loader {
        Loader {
            cm,
            env,
            globals: Default::default(),
        }
    }
}

impl Load for Loader {
    fn load(&self, f: &FileName) -> Result<ModuleData, Error> {
        let fm = match f {
            FileName::Real(path) => self.cm.load_file(path)?,
            m => unreachable!("{:?}", m),
        };

        let module = parse_file_as_module(
            &fm,
            Syntax::Es(EsConfig {
                export_default_from: true,
                ..Default::default()
            }),
            EsVersion::Es2020,
            None,
            &mut vec![],
        )
        .unwrap_or_else(|err| {
            let handler =
                Handler::with_tty_emitter(ColorConfig::Always, false, false, Some(self.cm.clone()));
            err.into_diagnostic(&handler).emit();
            panic!("failed to parse")
        });

        let helpers = Helpers::new(false);

        let mut inline_globals =
            inline_globals(self.env.clone(), self.globals.clone(), Default::default());

        let mut pass = simplifier(
            Mark::new(),
            SimplyConfig {
                ..Default::default()
            },
        );

        let module = helpers::HELPERS.set(&helpers, || {
            // Apply transforms (like decorators pass)
            let mut module = inline_globals.fold_module(module);

            loop {
                module = pass.fold_module(module);
                if !pass.changed() {
                    break;
                }
            }

            module
        });

        Ok(ModuleData {
            fm,
            module,
            helpers,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileLoader {
    virtual_path: PathBuf,
}

impl FileLoader {
    pub fn virtual_path(&self) -> &PathBuf {
        &self.virtual_path
    }
}

impl FileLoader {
    pub fn new(root: PathBuf) -> FileLoader {
        let vp = RelativePathBuf::from(NODE_MODULES_PREFIX);

        let virtual_path = vp.to_path(&root);

        FileLoader { virtual_path }
    }
}

impl SwcFileLoader for FileLoader {
    fn file_exists(&self, path: &std::path::Path) -> bool {
        todo!("exists {:?}", path);
    }

    fn abs_path(&self, path: &std::path::Path) -> Option<PathBuf> {
        todo!("abs {:?}", path);
    }

    fn read_file(&self, path: &std::path::Path) -> std::io::Result<String> {
        if path.starts_with(&self.virtual_path) {
            let module = path
                .to_string_lossy()
                .replace(&format!("{}/", self.virtual_path.to_string_lossy()), "");

            return Ok(format!(
                r#"
import * as cjs from '{}';
export {{ cjs as default }};
            "#,
                module
            ));
        }

        std::fs::read_to_string(path)
    }
}
