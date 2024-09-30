//! Base command-line interface.
use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

use super::{terminal::*, Action, Board, BoardElem, CellKind, Direction, MovableItem, Ui};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style, terminal, QueueableCommand,
};

const WON_MESSAGE: &str = "You won!";

#[derive(Debug)]
pub enum TuiError {
    IO(io::Error),
    MapTooLarge,
}

impl fmt::Display for TuiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TuiError::*;
        match self {
            IO(other) => write!(f, "IO error with terminal : {}", other),
            MapTooLarge => write!(f, "Board is bigger than screen."),
        }
    }
}

impl Error for TuiError {}

/// Base command-line interface.
/// The whole scene is reprinted each step and the input isn't real-time.
pub struct Tui;

impl Ui for Tui {
    fn initialize() -> Result<Self, Box<dyn Error>> {
        let res: Result<(), io::Error> = try {
            terminal::enable_raw_mode()?;

            let mut stdout = io::stdout();

            stdout
                .queue(cursor::Hide)?
                // .queue(terminal::Clear(terminal::ClearType::All))?
                .queue(terminal::EnterAlternateScreen)?
                .queue(terminal::SetTitle("Sooban"))?;

            stdout.flush()?;
        };
        res.map_err(|e| Box::new(TuiError::IO(e)))?;

        Ok(Tui)
    }

    fn cleanup(&self) -> Result<(), Box<dyn Error>> {
        let res: Result<(), io::Error> = try {
            let mut stdout = io::stdout();

            stdout
                .queue(cursor::Show)?
                .queue(terminal::LeaveAlternateScreen)?;
            stdout.flush()?;
            terminal::disable_raw_mode()?;
        };
        res.map_err(|e| Box::new(TuiError::IO(e)))?;

        Ok(())
    }

    fn get_input(&self) -> Result<Action, Box<dyn Error>> {
        let action = loop {
            let ev = event::read().map_err(|e| Box::new(TuiError::IO(e)))?;
            // io::stderr().execute(Print(format!("{:?}\n", ev)))?;
            match ev {
                Event::Resize(_, _) => break Action::Redraw,
                Event::Key(KeyEvent {
                    modifiers: KeyModifiers::NONE,
                    code,
                    ..
                }) => match code {
                    KeyCode::Char('q') => break Action::Quit,
                    KeyCode::Char('r') => break Action::ResetLevel,
                    KeyCode::Char('d') => break Action::Redraw,
                    KeyCode::Left => break Action::Movement(Direction::Left),
                    KeyCode::Right => break Action::Movement(Direction::Right),
                    KeyCode::Up => break Action::Movement(Direction::Up),
                    KeyCode::Down => break Action::Movement(Direction::Down),
                    _ => (),
                },
                Event::Key(KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('c'),
                    ..
                }) => break Action::Quit,
                _ => (),
            }
        };
        Ok(action)
    }

    fn display(
        &self,
        board: &Board,
        _last_move_result: Option<Option<(isize, isize)>>,
    ) -> Result<(), Box<dyn Error>> {
        let cols = u16::try_from(board.width()).map_err(|_| TuiError::MapTooLarge)?;
        let rows = u16::try_from(board.height()).map_err(|_| TuiError::MapTooLarge)?;

        let res: Result<(), io::Error> = try {
            let mut stdout = io::stdout();

            stdout.queue(terminal::Clear(terminal::ClearType::All))?;

            let (original_cols, original_rows) = terminal::size()?;

            if original_cols < cols || original_rows < rows {
                return Err(Box::new(TuiError::MapTooLarge));
            }

            let start_col = original_cols / 2 - cols / 2;
            let start_row = original_rows / 2 - rows / 2;

            for j in 0..rows {
                for i in 0..cols {
                    use CellKind::*;
                    use MovableItem::*;

                    let symbol = match board.get(i as isize, j as isize) {
                        BoardElem(_, Void) => SYMBOL_VOID,
                        BoardElem(_, Wall) => SYMBOL_WALL,
                        BoardElem(None, Floor) => SYMBOL_FLOOR,
                        BoardElem(None, Target) => SYMBOL_TARGET,
                        BoardElem(Some(Player), Floor) => SYMBOL_PLAYER,
                        BoardElem(Some(Crate(_)), Floor) => SYMBOL_CRATE,
                        BoardElem(Some(Player), Target) => SYMBOL_PLAYER_ON_TARGET,
                        BoardElem(Some(Crate(_)), Target) => SYMBOL_PLACED_CRATE,
                    };

                    stdout
                        .queue(cursor::MoveTo(start_col + i, start_row + j))?
                        .queue(style::Print(symbol))?;
                }
            }
            stdout.flush()?;
        };
        res.map_err(|e| Box::new(TuiError::IO(e)))?;

        Ok(())
    }

    fn won(&self) -> Result<(), Box<dyn Error>> {
        let res: Result<(), io::Error> = try {
            let mut stdout = io::stdout();

            /*
            let (cols, rows) = (WON_MESSAGE.chars().count() as u16, 1);
            let (original_cols, original_rows) = terminal::size()?;

            if original_cols < cols || original_rows < rows {
            */
            stdout
                .queue(cursor::MoveTo(0, 0))?
                .queue(style::Print(WON_MESSAGE))?;
            /*
            }

            let start_col = original_cols / 2 - cols / 2;
            let start_row = original_rows / 2 - rows / 2;

            for j in 0..rows {
                for i in 0..cols {
                    stdout
                        .queue(cursor::MoveTo(start_col + i, start_row + j))?
                        .queue(style::Print("I"))?;
                }
            }
            */
            stdout.flush()?;

            event::read()?;
        };
        res.map_err(|e| Box::new(TuiError::IO(e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
