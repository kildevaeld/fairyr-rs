use dale::Service;
use dale_http::{
    error::Error,
    prelude::{Set, With},
    Body, Outcome, Request, Response,
};
use dale_runtime::executor::Executor;
use fairy_core::Config;
use fairy_dev::create_resolver;
use futures_channel::oneshot;
use std::{future::Future, sync::Arc};

use crate::Options;

pub fn statics<B, E>(
    cfg: Arc<Options>,
) -> impl Service<Request<B>, Future = impl Future + Send, Output = Outcome<B>> + Clone
where
    B: Body + Send + 'static,
    E: Executor,
    E::Error: std::error::Error + Send + Sync + 'static,
{
    let config = Config {
        root: cfg.root.clone(),
        entry: cfg.entry.clone(),
        env: cfg.env.clone(),
        plugins: vec![],
    };

    let resolver = create_resolver(config).expect("create resolver");

    let resolver = Arc::new(resolver);

    let tp = threadpool::builder().build();

    move |req: Request<B>| {
        let resolver = resolver.clone();

        let path = req.uri().path().to_string();
        let (sx, rx) = oneshot::channel();

        tp.execute(move || {
            let ret = resolver.resolve(path);
            sx.send(ret).ok();
        });

        async move {
            let ret = rx.await;

            let bytes = match ret {
                Ok(ret) => match ret {
                    Ok(ret) => ret,
                    Err(err) => return Outcome::Failure(Error::new(err)),
                },
                Err(err) => return Outcome::Failure(Error::new(err)),
            };

            let resp = Response::<B>::with(bytes.content.to_bytes())
                .set(dale_http::headers::ContentType::from(bytes.mime));

            Outcome::Success(resp)
            // todo!()
        }
    }
}
