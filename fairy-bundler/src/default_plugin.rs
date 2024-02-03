use crate::{
    context::Context,
    plugin::{Data, ModuleId, Plugin, ResolvedId},
};
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_ecma_transforms::{fixer, hygiene};
use swc_ecma_visit::{Fold, FoldWith};

pub struct DefaultPlugin {}

impl Plugin<Context> for DefaultPlugin {
    fn resolve_id<'a>(&self, ctx: &Context, module: &'a str) -> Option<crate::plugin::ResolvedId> {
        let id = ResolvedId {
            id: ModuleId::Path(ctx.root().join(module)),
            external: false,
        };

        Some(id)
    }

    fn load(
        &self,
        ctx: &Context,
        module: &crate::plugin::ResolvedId,
    ) -> crate::Result<Option<crate::plugin::Data>> {
        if module.external {
            panic!("external");
        }

        let path = match &module.id {
            ModuleId::Path(p) => p,
            _ => panic!("no custom"),
        };

        let program = ctx.parse(path)?;

        Ok(Some(Data::Ast(program)))
    }

    fn transform(
        &self,
        id: &ModuleId,
        mut program: swc_ecma_ast::Program,
    ) -> crate::Result<swc_ecma_ast::Program> {
        program = hygiene().fold_program(program);
        program = fixer(None).fold_program(program);
        Ok(program)
    }
}
