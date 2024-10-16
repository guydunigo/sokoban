#![feature(try_blocks)]
//! Base data structures and functions to run a Sokoban-like game,
//! see [`game`] to start it.
use std::{error::Error, fmt, str::FromStr};

mod data;
use data::LevelParseError;
pub use data::{Board, BoardElem, CellKind, Direction};
mod ui;
#[cfg(feature = "ggez")]
pub use ui::game_ggez;
#[cfg(feature = "macroquad")]
pub use ui::game_macroquad;
use ui::Action;
pub use ui::{DisplayKind, Ui};

#[derive(Debug)]
pub enum GameError {
    LevelParseError(LevelParseError),
    UiError(Box<dyn Error>),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use GameError::*;
        match self {
            LevelParseError(e) => write!(f, "Error parsing level file: {}", e),
            UiError(e) => write!(f, "Error in the interface: {}", e),
        }
    }
}

impl Error for GameError {}

impl From<LevelParseError> for GameError {
    fn from(src: LevelParseError) -> Self {
        GameError::LevelParseError(src)
    }
}

/// Start the game by loading the level from the file content in `level_file`, and the display
/// selection in `disp_kind`.
pub fn game(disp_kind: DisplayKind, level: &str) -> Result<(), GameError> {
    let ui = ui::new(disp_kind).map_err(GameError::UiError)?;

    let res = game_loop(ui.as_ref(), level);

    // Whatever happened in the game, we close first.
    ui.cleanup().map_err(GameError::UiError)?;

    res
}

fn game_loop(ui: &dyn Ui, level: &str) -> Result<(), GameError> {
    let mut board = Board::from_str(level)?;
    loop {
        let res: Result<(), Box<dyn Error>> = try {
            ui.display(&board, None)?;
            loop {
                match ui.get_action(&board)? {
                    Action::Movement(dir) => {
                        let res = board.do_move_player(dir);

                        ui.display(&board, res)?;

                        // Si on a déplacé une caisse.
                        if let Some(Some(_)) = res {
                            if board.has_won() {
                                ui.won()?;
                                return Ok(());
                            }
                        }
                    }
                    Action::ResetLevel => board.reset(),
                    Action::Quit => return Ok(()),
                }
            }
        };
        res.map_err(GameError::UiError)?;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
