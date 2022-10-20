use std::collections::HashMap;

use locking::LockApi;

use crate::{bundler::Bundler, compiler::Compiler, content::Content, locket::Locket};

#[derive(Default)]
pub struct Externals {
    dependencies: Locket<HashMap<String, Content>>,
    bundler: Bundler,
}

impl Externals {
    // pub fn names<'a>(&'a self) -> impl Iterator<Item = &'a str> {
    //     self.dependencies.keys().map(|m| m.as_str())
    // }

    pub fn get(&self, name: &str) -> Option<Content> {
        self.dependencies.read().get(name).map(|m| m.clone())
    }

    pub fn try_get(&self, name: &str) -> anyhow::Result<Content> {
        match self.dependencies.read().get(name) {
            Some(ret) => Ok(ret.clone()),
            None => anyhow::bail!("module not found: {}", name),
        }
    }

    pub fn get_or_bundle(&self, compiler: &Compiler, name: &str) -> anyhow::Result<Content> {
        if let Some(found) = self.dependencies.read().get(name) {
            return Ok(found.clone());
        }

        let bundle = self.bundler.bundle(compiler, name)?;

        let content = Content::new(bundle.to_bytes(false)?);

        self.dependencies
            .write()
            .insert(name.to_string(), content.clone());

        Ok(content)
    }
}
