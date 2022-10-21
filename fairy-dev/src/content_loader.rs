use crate::{
    externals::Externals, loader::NODE_MODULES_PREFIX, transformers::EXTENSIONS, Compiler, Content,
    Error,
};
use relative_path::RelativePath;
use std::path::PathBuf;

pub struct Payload {
    pub mime: mime::Mime,
    pub content: Content,
}

#[cfg(feature = "concurrent")]
pub type ContentLoaderBox = Box<dyn ContentLoader + Send + Sync>;

#[cfg(not(feature = "concurrent"))]
pub type ContentLoaderBox = Box<dyn ContentLoader>;

pub trait ContentLoader {
    fn load(&self, path: &RelativePath) -> Result<Payload, Error>;
}

impl ContentLoader for ContentLoaderBox {
    fn load(&self, path: &RelativePath) -> Result<Payload, Error> {
        (&**self).load(path)
    }
}

pub struct ScriptLoader {
    compiler: Compiler,
    externals: Externals,
}

impl ScriptLoader {
    pub fn new(compiler: Compiler) -> ScriptLoader {
        ScriptLoader {
            compiler,
            externals: Externals::default(),
        }
    }

    pub fn resolve(&self, path: &RelativePath) -> Option<PathBuf> {
        if let Some(ext) = path.extension() {
            if !EXTENSIONS.contains(&ext) {
                return None;
            }
        }

        let fp = path.to_path(self.compiler.root());
        if fp.exists() {
            let meta = match fp.metadata() {
                Ok(ret) => ret,
                Err(_) => return None,
            };

            if meta.is_dir() {
                return None;
            }

            return Some(fp);
        }

        for ext in EXTENSIONS {
            let fp = path.with_extension(*ext).to_path(self.compiler.root());
            if fp.exists() {
                return Some(fp);
            }
        }

        None
    }
}

impl ContentLoader for ScriptLoader {
    fn load(&self, path: &RelativePath) -> Result<Payload, Error> {
        let content = if path.starts_with(NODE_MODULES_PREFIX) {
            let file_name = path.to_string().replace(NODE_MODULES_PREFIX, "");

            let bundle = match self.externals.get_or_bundle(&self.compiler, &file_name) {
                Ok(ret) => ret,
                Err(err) => {
                    log::error!("could not bundle '{}': {:?}", path, err);
                    return Err(err.into());
                }
            };
            bundle
        } else {
            let full_path = match self.resolve(path) {
                Some(path) => path,
                None => return Err(Error::NotFound),
            };

            Content::new(self.compiler.compile(full_path)?.code.into_bytes())
        };

        Ok(Payload {
            mime: mime::APPLICATION_JAVASCRIPT,
            content,
        })
    }
}

pub struct AssetLoader {
    root: PathBuf,
}

impl AssetLoader {
    pub fn new(root: PathBuf) -> AssetLoader {
        AssetLoader { root }
    }
}

impl ContentLoader for AssetLoader {
    fn load(&self, path: &RelativePath) -> Result<Payload, Error> {
        let ext = match path.extension() {
            Some(ext) => ext,
            None => return Err(Error::NotFound),
        };

        let fp = path.to_path(&self.root);

        if !fp.exists() {
            return Err(Error::NotFound);
        }

        let meta = fp.metadata()?;

        if meta.is_dir() {
            return Err(Error::NotFound);
        }

        let bytes = std::fs::read(fp)?;

        let mime = mime_guess::from_ext(ext).first_or_octet_stream();
        Ok(Payload {
            mime,
            content: Content::new(bytes),
        })
    }
}
