use std::{borrow::Cow, path::PathBuf};

use crate::Result;
use bytes::Bytes;
use relative_path::RelativePathBuf;
use swc_ecma_ast::Program;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleId {
    Custom(String),
    Path(PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Data {
    Source(Bytes),
    Ast(Program),
}

pub trait PluginContext {}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedId {
    pub id: ModuleId,
    pub external: bool,
}

pub trait Plugin<C> {
    fn resolve_id<'a>(&self, ctx: &C, module: &'a str) -> Option<ResolvedId> {
        None
    }

    fn load(&self, ctx: &C, module: &ResolvedId) -> Result<Option<Data>> {
        Ok(None)
    }

    fn transform(&self, id: &ModuleId, program: Program) -> Result<Program> {
        Ok(program)
    }
}

#[cfg(feature = "concurrent")]
pub type PluginBox<C> = Box<dyn Plugin<C> + Send + Sync>;

#[cfg(not(feature = "concurrent"))]
pub type PluginBox<C> = Box<dyn Plugin<C>>;
