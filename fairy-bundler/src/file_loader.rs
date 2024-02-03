use swc_common::FileLoader as SwcFileLoader;

use crate::core::Core;

pub struct FileLoader {
    core: Core,
}

impl FileLoader {
    pub fn new(core: Core) -> FileLoader {
        FileLoader { core }
    }
}

impl SwcFileLoader for FileLoader {
    fn file_exists(&self, path: &std::path::Path) -> bool {
        todo!()
    }

    fn abs_path(&self, path: &std::path::Path) -> Option<std::path::PathBuf> {
        todo!()
    }

    fn read_file(&self, path: &std::path::Path) -> std::io::Result<String> {
        todo!()
    }
}
