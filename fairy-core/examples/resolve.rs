use std::path::PathBuf;

use fairy_core::{ImportHint, Resolver, TargetEnv};

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let resolver = Resolver::new(PathBuf::from("fairy-http/examples/frontend").canonicalize()?);

    // resolver.resolve("src/main.tsx", "react");

    // resolver.resolve("src/main.tsx", "react-dom/client");

    let package = resolver.resolve(
        "node_modules/react-dome/index.js",
        "scheduler",
        ImportHint::Import,
        TargetEnv::Browser,
    );

    println!("package {:#?}", package);

    // resolver.resolve("src/main.tsx", "@stitches/react/test");

    Ok(())
}
