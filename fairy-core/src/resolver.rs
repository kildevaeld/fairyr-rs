use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use pathdiff::diff_paths;
use relative_path::{RelativePath, RelativePathBuf};
use serde::de::Expected;

use crate::{
    find_nearest_external, find_nearest_node_modules, find_nearest_package_json,
    module_type_from_ext,
    package::{Exports, PACKAGE_JSON},
    ModuleType, PackageJson, EXTENSIONS,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub pkgjson: PackageJson,
    pub root: PathBuf,
    pub entry: Entry,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub path: RelativePathBuf,
    pub kind: ModuleType,
}

fn is_path(path: &str) -> bool {
    path.starts_with("./") || path.starts_with("../") || path.starts_with("/")
}

fn real_id<'a>(id: &'a str) -> Option<(&'a str, Option<RelativePathBuf>)> {
    let ret = if id.starts_with("@") {
        let first = match id.find("/") {
            Some(first) => first,
            None => {
                log::debug!("invalid package name");
                return None;
            }
        };
        match &id[first + 1..].find("/") {
            Some(idx) => {
                let rest = &id[(first + idx) + 2..];
                let id = &id[..(first + idx + 1)];
                (id, Some(RelativePathBuf::from(format!("./{}", rest))))
            }
            None => (id, None),
        }
    } else {
        match id.find("/") {
            Some(idx) => {
                let rest = &id[idx + 1..];
                let id = &id[..idx];
                (id, Some(RelativePathBuf::from(format!("./{}", rest))))
            }
            None => (id, None),
        }
    };

    Some(ret)
}

pub struct Resolver {
    root: PathBuf,
}

impl Resolver {
    pub fn new(root: PathBuf) -> Resolver {
        Resolver { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn resolve(
        &self,
        base: impl AsRef<RelativePath>,
        id: &str,
        hint: ImportHint,
        target: TargetEnv,
    ) -> Option<Package> {
        if is_path(id) {
            self.resolve_path(base.as_ref(), &RelativePath::new(id))
        } else {
            self.resolve_module(base.as_ref(), id, hint, target)
        }
    }

    fn resolve_module(
        &self,
        base: &RelativePath,
        id: &str,
        hint: ImportHint,
        target: TargetEnv,
    ) -> Option<Package> {
        log::debug!("resolve external: {}", id);
        let fp_base = base.to_path(&self.root);

        let (real_id, sub_path) = match real_id(id) {
            Some(found) => found,
            None => {
                log::error!("external not found");
                return None;
            }
        };

        let node_modules = match find_nearest_external(&fp_base, real_id) {
            Some(node_modules) => node_modules,
            None => panic!("not node_modules"),
        };

        log::debug!(
            "resolved real id {} => {}, rest: {:?}",
            id,
            real_id,
            sub_path
        );

        let fp_pkg_root = node_modules.join(real_id);
        let fp_pkg_json = fp_pkg_root.join(PACKAGE_JSON);

        if !fp_pkg_json.exists() {
            log::debug!("external package {} not found at {:?}", id, fp_pkg_root)
        }

        log::debug!("found external package path: {:?}", fp_pkg_root);

        let pkg_json = match PackageJson::load(&fp_pkg_root) {
            Ok(ret) => ret,
            Err(err) => {
                log::error!("could not load package json: {}", err);
                return None;
            }
        };

        let ret = if let Some(path) = sub_path {
            pkg_json.resolve(&fp_pkg_root, &path, hint, target)
        } else {
            pkg_json.resolve_default(hint, target)
        };

        ret.map(|entry| Package {
            pkgjson: pkg_json,
            root: fp_pkg_root,
            entry,
        })
    }

    fn resolve_path(&self, base: &RelativePath, path: &RelativePath) -> Option<Package> {
        let parent = base.parent().unwrap_or_else(|| RelativePath::new("./"));

        let resolved_path = parent.join_normalized(path);

        let fp_path = resolved_path.to_path(&self.root);

        let (pkg_root, pkgjson) = match find_nearest_package_json(&fp_path) {
            Some(ret) => ret,
            None => {
                log::error!("no root level package.json");
                return None;
            }
        };

        let resolved_path = match diff_paths(&fp_path, &pkg_root) {
            Some(ret) => RelativePathBuf::from_path(ret).expect("path"),
            None => return None,
        };

        if fp_path.exists() {
            Some(Package {
                entry: Entry {
                    kind: module_type_from_ext(&resolved_path).unwrap_or(pkgjson.kind),
                    path: resolved_path,
                },
                root: pkg_root,
                pkgjson,
            })
        } else {
            for ext in EXTENSIONS {
                let resolved_path = resolved_path.with_extension(*ext);
                let fp_path = resolved_path.to_path(&self.root);

                if fp_path.exists() {
                    return Some(Package {
                        entry: Entry {
                            kind: module_type_from_ext(&resolved_path).unwrap_or(pkgjson.kind),
                            path: resolved_path,
                        },
                        root: pkg_root,
                        pkgjson,
                    });
                }
            }

            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportHint {
    Import,
    Require,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetEnv {
    Node,
    Browser,
}

impl TargetEnv {
    fn export_fields(&self, hint: ImportHint) -> &[&'static str] {
        match self {
            TargetEnv::Browser => match hint {
                ImportHint::Import => &["import", "browser", "require", "default"],
                ImportHint::Require => &["browser", "require", "default"],
            },
            TargetEnv::Node => match hint {
                ImportHint::Import => &["import", "node", "default"],
                ImportHint::Require => &["require", "node", "default"],
            },
        }
    }
}

fn get_one_of<'a>(
    obj: &'a HashMap<String, Exports>,
    oneof: &[&'static str],
) -> Option<&'a Exports> {
    for o in oneof {
        if let Some(found) = obj.get(*o) {
            return Some(found);
        }
    }

    None
}
impl PackageJson {
    fn resolve_exports(
        &self,
        exports: &Exports,
        hint: ImportHint,
        target_env: TargetEnv,
    ) -> Option<Entry> {
        match exports {
            Exports::Str(name) => {
                let path = RelativePath::new(name.as_str());
                let kind = module_type_from_ext(path).unwrap_or(self.kind);
                Some(Entry {
                    path: path.to_relative_path_buf(),
                    kind,
                })
            }
            Exports::Obj(obj) => {
                if let Some(current) = obj.get(".") {
                    self.resolve_exports(current, hint, target_env)
                } else {
                    let found = match get_one_of(&obj, target_env.export_fields(hint)) {
                        Some(found) => found,
                        None => return None,
                    };

                    let found = match found {
                        Exports::Str(str) => str,
                        _ => return self.resolve_exports(found, hint, target_env),
                    };

                    log::debug!("found package {}", found);

                    Some(Entry {
                        path: found.into(),
                        kind: module_type_from_ext(&RelativePath::new(found)).unwrap_or(self.kind),
                    })
                }
            }
        }
    }

    fn resolve_exports_path(
        &self,
        exports: &Exports,
        hint: ImportHint,
        target_env: TargetEnv,
        path: &RelativePath,
    ) -> Option<Entry> {
        match exports {
            Exports::Str(_) => return None,
            Exports::Obj(obj) => {
                if let Some(exports) = obj.get(path.as_str()) {
                    self.resolve_exports(exports, hint, target_env)
                } else {
                    None
                }
            }
        }
    }

    pub fn resolve_default(&self, hint: ImportHint, target_env: TargetEnv) -> Option<Entry> {
        if let Some(exports) = &self.exports {
            self.resolve_exports(exports, hint, target_env)
        } else {
            let (kind, path) = if hint == ImportHint::Require {
                (self.kind, self.main.as_ref().unwrap())
            } else {
                match self.kind {
                    ModuleType::Commonjs => self
                        .module
                        .as_ref()
                        .map(|m| (ModuleType::Esm, m))
                        .unwrap_or_else(|| (ModuleType::Commonjs, self.main.as_ref().unwrap())),
                    ModuleType::Esm => (ModuleType::Esm, self.main.as_ref().unwrap()),
                }
            };

            Some(Entry {
                kind,
                path: RelativePathBuf::from(path),
            })
        }
    }

    pub fn resolve(
        &self,
        root: &Path,
        path: &RelativePath,
        hint: ImportHint,
        target_env: TargetEnv,
    ) -> Option<Entry> {
        if let Some(exports) = &self.exports {
            if let Some(ret) = self.resolve_exports_path(exports, hint, target_env, path) {
                return Some(ret);
            }
        }

        if !path.to_logical_path(root).exists() {
            for ext in EXTENSIONS {
                let resolved_path = path.with_extension(*ext);
                let fp_path = resolved_path.to_path(&root);

                if fp_path.exists() {
                    return Some(Entry {
                        kind: module_type_from_ext(&resolved_path).unwrap_or(self.kind),
                        path: resolved_path,
                    });
                }
            }
            return None;
        }

        Some(Entry {
            kind: module_type_from_ext(path).unwrap_or(self.kind),
            path: path.to_relative_path_buf(),
        })
    }
}
