//! Base command-line interface.
use std::{
    error::Error,
    fmt,
    io::{self, Write},
    panic,
};

use super::{terminal::*, Action, Board, BoardElem, CellKind, Direction, MovableItem, Ui};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style, terminal, QueueableCommand,
};

const WON_MESSAGE_PADDING: u16 = 3;
const WON_MESSAGE_LN_1: &str = "You won!";
const WON_MESSAGE_LN_2: &str = "(Press any key to quit...)";

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

        panic::set_hook(Box::new(|panic_info| {
            Tui.cleanup()
                .expect("Couldn't clean terminal back to normal.");

            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                eprintln!("Panic occurred: {s:?}");
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                eprintln!("Panic occurred: {s:?}");
            } else {
                eprintln!("Panic occurred");
            }
        }));

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
                    KeyCode::Esc | KeyCode::Char('q') => break Action::Quit,
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

            let (term_cols, term_rows) = terminal::size()?;

            if term_cols < cols || term_rows < rows {
                return Err(Box::new(TuiError::MapTooLarge));
            }

            let start_col = term_cols / 2 - cols / 2;
            let start_row = term_rows / 2 - rows / 2;

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

            if board.has_won() {
                let start_right = start_col + cols + WON_MESSAGE_PADDING;
                /*
                let won_message_max_len = u16::try_from(
                    WON_MESSAGE_LN_1
                        .chars()
                        .count()
                        .max(WON_MESSAGE_LN_2.chars().count()),
                )
                .expect("Won message should fit in a u16.");
                */

                stdout
                    .queue(cursor::MoveTo(start_right, term_rows / 2 - 1))?
                    .queue(style::Print(WON_MESSAGE_LN_1))?
                    .queue(cursor::MoveToNextLine(2))?
                    .queue(cursor::MoveRight(start_right))?
                    .queue(style::Print(WON_MESSAGE_LN_2))?;
            }

            stdout.flush()?;
        };
        res.map_err(|e| Box::new(TuiError::IO(e)))?;

        Ok(())
    }

    fn won(&self) -> Result<(), Box<dyn Error>> {
        event::read().map_err(|e| Box::new(TuiError::IO(e)))?;
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
