//! Base data structures and functions to run a Sokoban-like game,
//! see [`game`] to start it.
use std::{error::Error, fmt, str::FromStr};

mod data;
use data::{Board, LevelParseError};
mod ui;
pub use ui::DisplayKind;
use ui::{new_ui, Action};

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
pub fn game(level_file: &str, disp_kind: DisplayKind) -> Result<(), GameError> {
    loop {
        let mut board = Board::from_str(level_file)?;
        let mut ui = new_ui(disp_kind);

        ui.display(&board, None).map_err(GameError::UiError)?;
        loop {
            match ui.get_input().map_err(GameError::UiError)? {
                Action::Movement(dir) => {
                    let res = board.do_move_player(dir);
                    ui.display(&board, res).map_err(GameError::UiError)?;

                    if let Some(Some(_)) = res {
                        if board.has_won() {
                            ui.won(&board).map_err(GameError::UiError)?;
                            return Ok(());
                        }
                    }
                }
                Action::ResetLevel => {
                    break;
                }
                Action::Quit => return Ok(()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
