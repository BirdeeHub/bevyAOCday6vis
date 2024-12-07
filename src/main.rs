use std::io::Result;
use bevy::prelude::*;
mod part1and2;
mod types;
fn main() -> Result<()> {
    part1and2::run()?;
    App::new().add_plugins(DefaultPlugins).run();
    Ok(())
}
