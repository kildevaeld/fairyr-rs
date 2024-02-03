use std::path::{Path, PathBuf};

use relative_path::{RelativePath, RelativePathBuf};
use swc_ecma_ast::Program;

use crate::{core::Core, Result};

pub struct Context {
    pub(crate) core: Core,
    parent: Option<RelativePathBuf>,
}

impl Context {
    pub fn new(core: Core, parent: Option<RelativePathBuf>) -> Context {
        Context { core, parent }
    }

    pub fn root(&self) -> PathBuf {
        self.core.root()
    }

    pub fn parse(&self, path: &Path) -> Result<Program> {
        self.core.parse(path)
    }
}
