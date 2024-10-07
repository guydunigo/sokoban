use std::error::Error;

use super::data::{Board, BoardElem, CellKind, Direction, MovableItem};

mod cli;
mod terminal;
use cli::Cli;
mod tui;
use tui::Tui;
mod ggez;
pub use ggez::game_ggez;
mod macroquad;
pub use macroquad::game_macroquad;
// mod gui;
// use gui::Gui;

/// How the game should be played.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayKind {
    /// Basic terminal prompt.
    CLI,
    /// Dynamic terminal display.
    TUI,
    // /// 2D graphics.
    // GUI,
}

/// Actions available through the UI
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Basic terminal prompt.
    Movement(Direction),
    /// Resets the caracter and crates layout.
    ResetLevel,
    /// Quit game
    Quit,
    // TODO: LoadLevel(String path)
}

/// Describes a generic interface to play the game.
pub trait Ui {
    /// All the setup needed for the UI : opening window, ...
    fn initialize() -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;

    /// All the cleaning needed for the UI : closing window, resetting terminal, ...
    // TODO: cleanup should take Self and destroy
    fn cleanup(self: Box<Self>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    /// Get last input from user. This is usually blocking.
    fn get_action(&self, board: &Board) -> Result<Action, Box<dyn Error>>;

    /// Updates the display based on the board provided and the result of the last move and if it
    /// pushed a crate.
    /// For instance, in `last_move_result` is `None`, it means the player couldn't move, so the
    /// display might not need to be updated, but might trigger a sound.
    /// See [`Board::do_move_player`] for more information on `last_move_result`.
    ///
    /// It can directly check and react on [`Board::has_won`].
    fn display(
        &self,
        board: &Board,
        last_move_result: Option<Option<(u32, u32)>>,
    ) -> Result<(), Box<dyn Error>>;

    /// The game is won and will quit the game when this function returns.
    fn won(&self) -> Result<(), Box<dyn Error>>;
}

pub fn new(kind: DisplayKind) -> Result<Box<dyn Ui>, Box<dyn Error>> {
    use DisplayKind::*;

    Ok(match kind {
        CLI => Box::new(Cli::initialize()?),
        TUI => Box::new(Tui::initialize()?),
        // GUI -> Box::new(Gui::new()),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
