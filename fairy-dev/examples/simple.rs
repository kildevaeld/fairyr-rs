use std::{collections::HashMap, path::PathBuf};

use fairy_core::Config;
use fairy_dev::create_resolver;
use relative_path::RelativePathBuf;

fn main() -> anyhow::Result<()> {
    let mut env = HashMap::default();

    env.insert("NODE_ENV".into(), "development".into());

    let config = Config {
        root: PathBuf::from("fairy-http/examples/frontend"),
        entry: RelativePathBuf::from("/src/main.tsx"),
        env,
        plugins: Vec::default(),
    };

    let resolver = create_resolver(config)?;

    // let package = resolver.compiler.resolve("react")?;

    // println!("{:#?}", package);

    let names = resolver.resolve("/src/main.tsx")?;
    println!("{}", names.content.to_string()?);

    // let deps = app.dependencies()?;

    // let compiler = Compiler::new(app.root.clone());

    // let dep = deps.iter().find(|m| &m.name == "react-dom").unwrap();

    // let dep = compiler.bundle(dep)?;

    // println!("{}", dep.to_string(false)?);

    Ok(())
}
