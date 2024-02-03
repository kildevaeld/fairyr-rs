use std::path::{Path, PathBuf};

use crate::{
    chunk::Chunk,
    core::Core,
    file_loader::FileLoader,
    import_resolver::{ImportResolver, ImportResolver2},
    module_graph::{ChunkId, ModuleGraph},
    plugin::{Data, ModuleId, ResolvedId},
    Config, Result,
};
use daggy::Walker;
use generational_arena::Arena;
use indexmap::IndexSet;
use swc_common::{sync::Lrc, FilePathMapping, SourceMap, DUMMY_SP};
use swc_ecma_ast::{EsVersion, Module, Program};
use swc_ecma_codegen::{
    text_writer::{omit_trailing_semi, JsWriter, WriteJs},
    Emitter,
};
use swc_ecma_parser::{parse_file_as_program, Syntax, TsConfig};
use swc_ecma_visit::{VisitMut, VisitMutWith};
pub struct Bundler {
    core: Core,
    modules: ModuleGraph,
}

impl Bundler {
    pub fn new(root: impl AsRef<Path>, config: Config) -> Result<Bundler> {
        Ok(Bundler {
            core: Core::new(root.as_ref(), config)?,
            modules: ModuleGraph::new(),
        })
    }

    pub fn bundle(&mut self) -> Result<()> {
        let resolved_id = match self.core.resolve_entry()? {
            Some(resolve) => resolve,
            None => panic!("enrty point not found"),
        };

        let tree = self.resolve_tree(&resolved_id)?;

        self.modules.print();

        // for next in self.modules.dag.children(tree.0).iter(&self.modules.dag) {
        //     println!("next {:?}", self.modules.dag[next.1]);
        // }
        let out = self.modules.recursive(tree);

        for next in out {
            println!("{:?}", self.modules.dag[next.0]);
        }

        // let mut order = IndexSet::<ChunkId>::default();

        // self.sort_import(tree, &mut order)?;

        let mut module = Module {
            span: DUMMY_SP,
            body: vec![],
            shebang: None,
        };

        // for item in out {
        //     let chunk = self.modules.get(item);
        //     let m = match &chunk.program {
        //         Program::Module(m) => m.clone(),
        //         _ => panic!("program not supported"),
        //     };
        //     println!("id {:?}", chunk.id);
        //     module.body.extend(m.body);
        // }

        ImportResolver2::default().visit_mut_module(&mut module);

        // let mut buf = vec![];

        let minify = false;

        let mut buf = vec![];

        {
            let wr = JsWriter::new(self.core.cm(), "\n", &mut buf, None);
            let mut emitter = Emitter {
                cfg: swc_ecma_codegen::Config {
                    minify,
                    ..Default::default()
                },
                cm: self.core.cm(),
                comments: None,
                wr: if minify {
                    Box::new(omit_trailing_semi(wr)) as Box<dyn WriteJs>
                } else {
                    Box::new(wr) as Box<dyn WriteJs>
                },
            };

            emitter.emit_module(&module)?;
        }

        std::fs::write("output.js", buf);

        Ok(())
    }

    fn resolve_tree_from_lookup(&mut self, lookup: &str) -> Result<ChunkId> {
        let resolved_id = match self.core.resolve_id(lookup)? {
            Some(resolve) => resolve,
            None => panic!("enrty point not found"),
        };

        self.resolve_tree(&resolved_id)
    }

    fn resolve_tree(&mut self, id: &ResolvedId) -> Result<ChunkId> {
        if let Some(id) = self.modules.contains(&id.id) {
            return Ok(id);
        }

        let data = match self.core.load(id)? {
            Some(data) => data,
            None => panic!("no data"),
        };

        let mut program = match data {
            Data::Ast(program) => program,
            Data::Source(source) => panic!("not source"),
        };

        program = self.core.transform(&id.id, program)?;

        let mut import = ImportResolver::default();

        import.visit_mut_program(&mut program);

        let id = self.modules.register(&id.id);

        

        for import in import.0 {
            let i = self.resolve_tree_from_lookup(&import)?;
            if let Err(err) = self.modules.import(id, i) {
                panic!("err {}", err);
            }
        }

        // let chunk = Chunk {
        //     id: id.id.clone(),
        //     imports,
        //     program,
        //     is_entry: false,
        // };

        // let id = self.chunks.insert(chunk);

        Ok(id)
    }
}
