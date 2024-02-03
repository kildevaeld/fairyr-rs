use swc_ecma_ast::Program;

use crate::plugin::ModuleId;

pub type ChunkId = generational_arena::Index;

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub id: ModuleId,
    pub imports: Vec<ChunkId>,
    pub program: Program,
    pub is_entry: bool,
}
