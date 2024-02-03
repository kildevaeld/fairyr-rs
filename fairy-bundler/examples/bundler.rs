use fairy_bundler::{Bundler, Config, Result};

fn main() -> Result<()> {
    let cfg = Config {
        entry: "main.js".into(),
    };

    let mut bundler = Bundler::new("fairy-bundler/examples", cfg)?;

    bundler.bundle()?;

    Ok(())
}
