use std::path::PathBuf;

use anyhow::bail;
use fairy_core::{ImportHint, Package, TargetEnv};
use pathdiff::diff_paths;
use relative_path::RelativePathBuf;
use swc_bundler::Resolve;
use swc_common::FileName;

pub struct Resolver {
    i: fairy_core::Resolver,
}

impl Resolver {
    pub fn new(root: PathBuf) -> Resolver {
        Resolver {
            i: fairy_core::Resolver::new(root),
        }
    }

    pub fn resolve_external(&self, name: &str) -> Option<Package> {
        self.i
            .resolve("./main.js", name, ImportHint::Import, TargetEnv::Browser)
    }
}

impl Resolve for Resolver {
    fn resolve(&self, base: &FileName, module_specifier: &str) -> Result<FileName, anyhow::Error> {
        let path = match base {
            FileName::Real(real) => {
                if real.is_absolute() {
                    let diff = match diff_paths(real, &self.i.root()) {
                        Some(diff) => diff,
                        None => bail!("invalid path"),
                    };
                    RelativePathBuf::from_path(diff)?
                } else {
                    RelativePathBuf::from_path(real)?
                }
            }
            _ => bail!("only real supported"),
        };

        let package = self.i.resolve(
            &path,
            module_specifier,
            ImportHint::Import,
            TargetEnv::Browser,
        );

        match package {
            Some(package) => {
                let file_name = FileName::Real(package.entry.path.to_logical_path(&package.root));

                Ok(file_name)
            }
            None => {
                bail!("module not found: {} from base: {}", module_specifier, path)
            }
        }
    }
}
