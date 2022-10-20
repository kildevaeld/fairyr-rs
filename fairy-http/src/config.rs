use std::path::PathBuf;

use fairy_core::Environ;
use relative_path::RelativePathBuf;

pub struct RenderRequest {
    pub scripts: Vec<String>,
    pub links: Vec<String>,
    pub content: Option<String>,
}

// #[derive(Debug)]
pub struct Options {
    pub root: PathBuf,
    pub entry: RelativePathBuf,
    pub env: Environ,
    pub template: Box<dyn Fn(RenderRequest) -> Vec<u8> + Send + Sync>,
}
