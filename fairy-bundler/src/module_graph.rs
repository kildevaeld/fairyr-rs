// use petgraph::{
//     data::Build,
//     dot::{Config, Dot},
//     graph::NodeIndex,
//     Graph,
// };

use daggy::{
    petgraph::dot::{Config as DotConfig, Dot},
    Dag, EdgeIndex, NodeIndex, RecursiveWalk, Walker, WouldCycle,
};
use swc_ecma_ast::Program;

use crate::plugin::ModuleId;

#[derive(Debug)]
pub struct Chunk {
    id: ModuleId,
    program: Program,
}

#[derive(Debug)]
pub enum Relation {
    Import,
}

#[derive(Debug, Clone, Copy)]
pub struct ChunkId(pub NodeIndex);

pub struct ModuleGraph {
    pub dag: Dag<Chunk, Relation>,
}

impl ModuleGraph {
    pub fn new() -> ModuleGraph {
        ModuleGraph {
            dag: Dag::default(),
        }
    }

    pub fn contains(&self, id: &ModuleId) -> Option<ChunkId> {
        self.dag
            .graph()
            .node_indices()
            .find(|idx| {
                if &self.dag[*idx].id == id {
                    true
                } else {
                    false
                }
            })
            .map(ChunkId)
    }

    pub fn register(&mut self, id: &ModuleId, program: Program) -> ChunkId {
        let found = self.dag.graph().node_indices().find(|idx| {
            if &self.dag[*idx].id == id {
                true
            } else {
                false
            }
        });

        if let Some(found) = found {
            return ChunkId(found);
        }

        ChunkId(self.dag.add_node(Chunk {
            id: id.clone(),
            program,
        }))
    }

    pub fn recursive(&self, id: ChunkId) -> Vec<ChunkId> {
        let mut out = Vec::default();

        self._recursive(id, &mut out);

        out
    }

    pub fn _recursive(&self, id: ChunkId, items: &mut Vec<ChunkId>) {
        self.dag.children(id.0).iter(&self.dag).for_each(|item| {
            if items.iter().find(|i| i.0 == item.1).is_none() {
                self._recursive(ChunkId(item.1), items);
            }
        });
        items.push(id);
    }

    pub fn import(
        &mut self,
        importee: ChunkId,
        target: ChunkId,
    ) -> Result<EdgeIndex<u32>, WouldCycle<Relation>> {
        self.dag.update_edge(importee.0, target.0, Relation::Import)
    }

    pub fn print(&self) {
        std::fs::write(
            "graph.dot",
            format!(
                "{:?}",
                Dot::with_config(&&self.dag, &[DotConfig::EdgeNoLabel])
            ),
        )
        .expect("write");
    }

    pub fn get(&self, id: ChunkId) -> &Chunk {
        &self.dag[id.0]
    }

    pub fn get_mut(&mut self, id: ChunkId) -> &mut Chunk {
        &mut self.dag[id.0]
    }
}
