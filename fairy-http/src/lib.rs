use std::{future::Future, sync::Arc};

use dale::{Service, ServiceExt};
use dale_http::{prelude::Modifier, Body, Outcome, Request, Response};

mod config;
mod index;
mod statics;

pub use config::*;
use dale_runtime::executor::Executor;

pub fn create_routes<B, E>(
    cfg: Arc<Options>,
) -> impl Service<Request<B>, Future = impl Future + Send, Output = Outcome<B>> + Clone
where
    B: Body + Modifier<Response<B>> + Sync + Send + 'static,
    E: Executor + 'static,
    E::Error: std::error::Error + Send + Sync + 'static,
{
    dale_http::fs::dir(cfg.public.to_path(&cfg.root))
        .or(statics::statics::<B, E>(cfg.clone()))
        .unify()
        .or(index::index(cfg))
        .unify()
}
