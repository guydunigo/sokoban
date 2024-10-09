//! Wrapper for hot-reloadable plugin.
use fyrox_sokoban::{fyrox::plugin::Plugin, Game};

#[no_mangle]
pub fn fyrox_plugin() -> Box<dyn Plugin> {
    Box::new(Game::default())
}
