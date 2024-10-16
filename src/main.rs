extern crate sokoban;

use std::{env::args, fs::read_to_string};

const DEFAULT_LEVEL_FILENAME: &str = "./map.txt";

fn main() {
    let arg1 = args().nth(1);
    let level_filename = arg1.as_ref().map_or(DEFAULT_LEVEL_FILENAME, |f| &f[..]);

    let level = match read_to_string(level_filename) {
        Ok(l) => l,
        Err(err) => {
            eprintln!("Could not open file `{}`: {}", level_filename, err);
            return;
        }
    };

    #[cfg(not(any(
        feature = "tui",
        feature = "ggez",
        feature = "macroquad",
        feature = "bevy"
    )))]
    match sokoban::game(sokoban::DisplayKind::CLI, &level[..]) {
        Ok(()) => (),
        Err(err) => eprintln!("Game exited with following error :\n{}", err),
    }

    #[cfg(feature = "tui")]
    match sokoban::game(sokoban::DisplayKind::TUI, &level[..]) {
        Ok(()) => (),
        Err(err) => eprintln!("Game exited with following error :\n{}", err),
    }

    #[cfg(all(feature = "ggez", not(feature = "macroquad")))]
    match sokoban::game_ggez(&level[..]) {
        Ok(()) => (),
        Err(err) => eprintln!("Game exited with following error :\n{}", err),
    }

    #[cfg(feature = "macroquad")]
    sokoban::game_macroquad(&level[..]);
}
