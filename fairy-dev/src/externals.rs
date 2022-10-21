use std::collections::HashMap;

use locking::LockApi;

use crate::{bundler::Bundler, compiler::Compiler, content::Content, locket::Locket};

#[derive(Default)]
pub struct Externals {
    dependencies: Locket<HashMap<String, Content>>,
    bundler: Bundler,
}

impl Externals {
    pub fn get_or_bundle(&self, compiler: &Compiler, name: &str) -> anyhow::Result<Content> {
        if let Some(found) = self.dependencies.read().get(name) {
            return Ok(found.clone());
        }

        log::debug!("bundle {}", name);

        let bundle = self.bundler.bundle(compiler, name)?;

        let content = Content::new(bundle.to_bytes(false)?);

        self.dependencies
            .write()
            .insert(name.to_string(), content.clone());

        Ok(content)
    }
}
