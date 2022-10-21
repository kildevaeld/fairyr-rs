use relative_path::RelativePath;

use crate::{package::PACKAGE_JSON, ModuleType, PackageJson};
use std::path::{Path, PathBuf};

pub static NODE_MODULES: &'static str = "node_modules";

pub static EXTENSIONS: &'static [&'static str] = &["js", "jsx", "mjs", "ts", "tsx", "cjs", "mjs"];

pub fn find_package_root(path: &Path) -> Option<PathBuf> {
    let mut parent = path.parent();
    while let Some(p) = parent {
        let pkg = p.join(PACKAGE_JSON);
        if pkg.is_file() {
            return Some(p.to_path_buf());
        }
        parent = p.parent();
    }
    None
}

pub fn find_nearest_package_json(path: &Path) -> Option<(PathBuf, PackageJson)> {
    match find_package_root(path) {
        Some(root) => PackageJson::load(&root).map(|pkg| (root, pkg)).ok(),
        None => None,
    }
}

pub fn find_nearest_node_modules(path: &Path) -> Option<PathBuf> {
    let mut parent = Some(path);

    while let Some(path) = parent {
        let node_modules_path = path.join(NODE_MODULES);
        if node_modules_path.exists() {
            return Some(node_modules_path);
        }

        parent = path.parent();
    }

    None
}

pub fn find_nearest_external(path: &Path, id: &str) -> Option<PathBuf> {
    let mut parent = Some(path);

    while let Some(path) = parent {
        let node_modules_path = path.join(NODE_MODULES);
        if node_modules_path.exists() {
            if node_modules_path.join(id).exists() {
                return Some(node_modules_path);
            } else {
                // pri //ntln!("not found {:?} {:?}", path, node_modules_path.join(id));
            }
        }

        parent = path.parent();
    }

    None
}

pub fn find_nearest_package(path: &Path, package: &str) -> Option<PathBuf> {
    let mut parent = Some(path);

    while let Some(path) = parent {
        let node_modules_path = path.join(NODE_MODULES).join(package);
        if node_modules_path.exists() {
            return Some(node_modules_path);
        }

        parent = path.parent();
    }

    None
}

pub fn module_type_from_ext(path: &RelativePath) -> Option<ModuleType> {
    match path.extension() {
        Some("cjs") => Some(ModuleType::Commonjs),
        Some("mjs") => Some(ModuleType::Esm),
        _ => None,
    }
}
