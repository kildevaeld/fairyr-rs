use std::{collections::HashMap, path::PathBuf};

use fairy_core::Config;
use fairy_dev::{create_resolver, Resolver};
use relative_path::RelativePathBuf;

use swc_bundler::Resolve;
use swc_common::FileName;
use swc_ecma_loader::resolvers::node::NodeModulesResolver;

fn test_node_resolver() -> anyhow::Result<()> {
    let node = NodeModulesResolver::default();

    let found = node.resolve(
        &FileName::Real("fairy-http/examples/frontend/src/main.tsx".into()),
        "react",
    )?;
    println!("node found {}", found);

    Ok(())
}

fn tesst_resolver() -> anyhow::Result<()> {
    let resolver = Resolver::new(PathBuf::from("fairy-http/examples/frontend").canonicalize()?);

    let found = resolver.resolve(
        &FileName::Real("fairy-http/examples/frontend/src/main.tsx".into()),
        "react",
    )?;
    println!("found {}", found);

    Ok(())
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let mut env = HashMap::default();

    env.insert("NODE_ENV".into(), "development".into());

    let config = Config {
        root: PathBuf::from("fairy-http/examples/frontend").canonicalize()?,
        entry: RelativePathBuf::from("./src/main.tsx"),
        env,
        plugins: Vec::default(),
    };

    // test_node_resolver()?;
    // tesst_resolver()?;

    let resolver = create_resolver(config)?;

    // let package = resolver.compiler.resolve("react")?;

    // println!("{:#?}", package);

    let names = resolver.resolve("/node_modules/.fairy/prop-types")?;

    // println!("{}", names.content.to_string()?);

    // let deps = app.dependencies()?;

    // let compiler = Compiler::new(app.root.clone());

    // let dep = deps.iter().find(|m| &m.name == "react-dom").unwrap();

    // let dep = compiler.bundle(dep)?;

    // println!("{}", dep.to_string(false)?);

    Ok(())
}
