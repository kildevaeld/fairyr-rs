use std::{future::Future, sync::Arc};

use dale::{Service, ServiceExt};
use dale_http::{
    filters,
    prelude::{Set, With},
    Body, Outcome, Request, Response,
};

use super::{Options, RenderRequest};

pub fn index<B>(
    cfg: Arc<Options>,
) -> impl Service<Request<B>, Future = impl Future + Send, Output = Outcome<B>> + Clone
where
    B: Body + Send + 'static,
{
    filters::get::<B>()
        .and(filters::path("/"))
        .and(dale::filters::state(cfg.clone()).err_into::<dale_http::error::Error>())
        .and_then(|req: Arc<Options>| async move {
            //

            let template = (req.template)(RenderRequest {
                scripts: vec![req.entry.to_string()],
                links: vec![],
                content: None,
            });

            let resp = Response::<B>::with(template).set(dale_http::headers::ContentType::html());

            dale_http::Result::Ok(resp)
        })
        .err_into()
        .unpack_one()
}
