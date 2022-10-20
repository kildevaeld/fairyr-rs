mod config;
pub mod package;

use std::path::{Path, PathBuf};

pub use self::{
    config::*,
    package::{ModuleType, PackageJson},
};

pub fn find_package_root(path: &Path) -> Option<PathBuf> {
    let mut parent = path.parent();
    while let Some(p) = parent {
        let pkg = p.join(package::PACKAGE_JSON);
        if pkg.is_file() {
            return Some(p.to_path_buf());
        }
        parent = p.parent();
    }
    None
}
