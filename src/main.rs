extern crate sokoban;

use std::fs::read_to_string;

const LEVEL_FILENAME: &str = "./map.txt";

fn main() {
    let level = match read_to_string(LEVEL_FILENAME) {
        Ok(l) => l,
        Err(err) => {
            eprintln!("Could not open file `{}`: {}", LEVEL_FILENAME, err);
            return;
        }
    };

    match sokoban::game(&level[..], sokoban::DisplayKind::CLI) {
        Ok(()) => (),
        Err(err) => eprintln!("Game exited.\n{}", err),
    }
}
