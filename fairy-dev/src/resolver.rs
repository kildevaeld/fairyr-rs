use relative_path::RelativePath;

use crate::{
    compiler::Compiler,
    content_loader::{AssetLoader, ContentLoaderBox, Payload, ScriptLoader},
};

pub struct Resolver {
    pub loaders: Vec<ContentLoaderBox>,
}

impl Resolver {
    pub fn new(compiler: Compiler) -> Resolver {
        let root = compiler.root().to_path_buf();

        let loaders = vec![
            Box::new(ScriptLoader::new(compiler)) as ContentLoaderBox,
            Box::new(AssetLoader::new(root)),
        ];

        Resolver { loaders }
    }
}

impl Resolver {
    pub fn resolve(&self, path: impl AsRef<RelativePath>) -> anyhow::Result<Payload> {
        let path = path.as_ref();

        let mut last_err = None;

        for loader in self.loaders.iter() {
            match loader.load(path) {
                Ok(ret) => return Ok(ret),
                Err(err) => {
                    last_err = Some(err);
                }
            };
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("could not resolve")))
    }
}
