use std::{collections::HashMap, path::PathBuf, sync::Arc};

use dale_http::prelude::*;
use dale_runtime::Tokio;
use fairy_http::{create_routes, Options, RenderRequest};
use hyper::Server;
use relative_path::RelativePathBuf;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let mut env = HashMap::default();
    env.insert("NODE_ENV".into(), "development".into());

    let cfg = Options {
        root: PathBuf::from("./fairy-http/examples/frontend").canonicalize()?,
        entry: RelativePathBuf::from("/src/main.tsx"),
        env,
        template: Box::new(|req| {
            //
            Home { cfg: &req }.to_string().into_bytes()
        }),
    };

    let service = create_routes::<_, Tokio>(Arc::new(cfg));

    Server::bind(&addr)
        .serve(dale_http::hyper::make(service))
        .await;

    Ok(())
}

markup::define!(
    Home<'a>(cfg: &'a RenderRequest) {
        @markup::doctype()
        html {
            head {
                style {
                    "body { background: #fafbfc; }"
                    "#main { padding: 2rem; }"
                }
                script {
                    "window.process = { env: {} };"
                }
            }
            body {
                #root {}
                @for script in &cfg.scripts {
                    script["type" = "module", "src" = script] {}
                }
            }
        }
    }
);
