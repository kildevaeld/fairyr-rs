use dale::ServiceExt;
use dale_runtime::Tokio;
use fairy_http::{create_routes, Options, RenderRequest};
use hyper::Server;
use std::sync::Arc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init();

    let addr = ([127, 0, 0, 1], 3000).into();

    let cfg = Options::build("./fairy-http/examples/frontend")
        .entry("./src/main.tsx")
        .template(|req| dale_http::Result::Ok(Home { cfg: &req }.to_string().into_bytes()))
        .with_env("NODE_ENV", "development")
        .build()?;

    let service = create_routes::<_, Tokio>(Arc::new(cfg)).map_err(|err| {
        println!("ERROR {}", err);
        err
    });

    let ret = Server::bind(&addr)
        .serve(dale_http::hyper::make(service))
        .await;

    match ret {
        Ok(_) => return Ok(()),
        Err(err) => {
            //
            println!("err {}", err);
        }
    }

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
