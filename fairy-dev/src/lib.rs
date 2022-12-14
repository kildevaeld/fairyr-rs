mod bundler;
pub mod compiler;
mod content;
mod content_loader;
mod externals;
mod fairy;
pub mod loader;
mod locket;
mod resolver;
mod transformers;
mod error;

pub use self::{
    bundler::{Bundle, Bundler},
    compiler::Compiler,
    content::Content,
    fairy::FairyDev,
    resolver::Resolver,
    error::*,
};

use swc_atoms::JsWord;
use swc_common::sync::Lrc;
use swc_ecma_ast::{Expr, Lit};

pub fn create_resolver(config: fairy_core::Config) -> anyhow::Result<FairyDev> {
    let env = config
        .env
        .into_iter()
        .map(|(k, v)| (JsWord::from(k), Expr::Lit(Lit::Str(v.into()))))
        .collect();

    let env = Lrc::new(env);

    let compiler = Compiler::new(config.root.clone(), env);

    Ok(FairyDev::new(compiler))
}
