use relative_path::RelativePath;

use crate::{
    compiler::Compiler,
    content_loader::{AssetLoader, ContentLoaderBox, Payload, ScriptLoader},
    Error,
};

pub struct FairyDev {
    pub loaders: Vec<ContentLoaderBox>,
}

impl FairyDev {
    pub fn new(compiler: Compiler) -> FairyDev {
        let root = compiler.root().to_path_buf();

        let loaders = vec![
            Box::new(ScriptLoader::new(compiler)) as ContentLoaderBox,
            Box::new(AssetLoader::new(root)),
        ];

        FairyDev { loaders }
    }
}

impl FairyDev {
    pub fn resolve(&self, path: impl AsRef<RelativePath>) -> Result<Payload, Error> {
        let path = path.as_ref();

        for loader in self.loaders.iter() {
            match loader.load(path) {
                Ok(ret) => return Ok(ret),
                Err(err) => match err {
                    Error::NotFound => {
                        continue;
                    }
                    err => return Err(err),
                },
            };
        }

        Err(Error::NotFound)
    }
}
