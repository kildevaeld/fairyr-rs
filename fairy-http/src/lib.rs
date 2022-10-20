use std::{future::Future, sync::Arc};

use dale::{Service, ServiceExt};
use dale_http::{Body, Outcome, Request};

mod config;
mod index;
mod statics;

pub use config::*;
use dale_runtime::executor::Executor;

pub fn create_routes<B, E>(
    cfg: Arc<Options>,
) -> impl Service<Request<B>, Future = impl Future + Send, Output = Outcome<B>> + Clone
where
    B: Body + Send + 'static,
    E: Executor + 'static,
    E::Error: std::error::Error + Send + Sync + 'static,
{
    index::index(cfg.clone())
        .or(statics::statics::<B, E>(cfg))
        .unify()
}
