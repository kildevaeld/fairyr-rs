use std::path::{Path, PathBuf};

use locking::LockApi;
use relative_path::{RelativePath, RelativePathBuf};
use swc_common::{
    comments::{Comments, SingleThreadedComments},
    sync::Lrc,
    FileName, SourceMap,
};
use swc_ecma_ast::Program;
use swc_ecma_parser::{lexer::Lexer, parse_file_as_program, EsConfig, Parser, StringInput, Syntax};

use crate::{
    context::Context,
    default_plugin::DefaultPlugin,
    module_graph::ModuleGraph,
    plugin::{Data, ModuleId, PluginBox, ResolvedId},
    sync::Lock,
    Config, Result,
};

struct CoreInner {
    root: PathBuf,
    entry: RelativePathBuf,
    plugins: Vec<PluginBox<Context>>,
    cm: Lrc<SourceMap>,
    comments: SingleThreadedComments,
}

#[derive(Clone)]
pub struct Core(Lock<CoreInner>);

impl Core {
    pub fn new(root: &Path, config: Config) -> Result<Core> {
        let cm = Lrc::new(SourceMap::default());
        Ok(Core(Lock::new(CoreInner {
            root: root.canonicalize()?,
            cm: cm.clone(),
            entry: config.entry.into(),
            plugins: vec![Box::new(DefaultPlugin {})],
            comments: SingleThreadedComments::default(),
        })))
    }

    pub fn cm(&self) -> Lrc<SourceMap> {
        self.0.read().cm.clone()
    }

    pub fn root(&self) -> PathBuf {
        self.0.read().root.clone()
    }

    pub fn resolve_entry<'a>(&'a self) -> Result<Option<ResolvedId>> {
        let entry = self.0.read().entry.to_string();
        self.resolve_id(&entry)
    }

    pub fn resolve_id<'a>(&self, id: &'a str) -> Result<Option<ResolvedId>> {
        let core = self.0.read();

        let ctx = Context::new(self.clone(), None);

        for plugin in &core.plugins {
            if let Some(found) = plugin.resolve_id(&ctx, id) {
                return Ok(Some(found));
            }
        }

        Ok(None)
    }

    pub fn load(&self, id: &ResolvedId) -> Result<Option<Data>> {
        let core = self.0.read();

        let ctx = Context::new(self.clone(), None);

        for plugin in &core.plugins {
            if let Some(found) = plugin.load(&ctx, id)? {
                return Ok(Some(found));
            }
        }

        Ok(None)
    }

    pub fn transform(&self, id: &ModuleId, mut program: Program) -> Result<Program> {
        let core = self.0.read();
        for plugin in &core.plugins {
            program = match plugin.transform(id, program) {
                Ok(ret) => ret,
                Err(err) => return Err(err),
            }
        }

        Ok(program)
    }

    // Utils

    pub fn parse<'a>(&self, path: &Path) -> Result<Program> {
        let core = self.0.read();

        let fm = core.cm.load_file(path)?;

        let mut parser = Parser::new(
            Syntax::Es(EsConfig {
                ..Default::default()
            }),
            StringInput::from(fm.as_ref()),
            Some(&core.comments),
        );

        Ok(parser.parse_program()?)
    }

    pub fn parse_source<'a>(&self, name: &'a str, source: &'a str) -> Result<Program> {
        let core = self.0.read();

        let fm = core.cm.new_source_file(FileName::Anon, source.to_string());

        let mut parser = Parser::new(
            Syntax::Es(EsConfig {
                ..Default::default()
            }),
            StringInput::from(fm.as_ref()),
            Some(&core.comments),
        );

        Ok(parser.parse_program()?)
    }

    // fn each<F, A, R>(&self, args: A, func: F) -> R where F: Fn(&Context, A) -> R {

    // }
}
