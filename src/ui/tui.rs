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
    style::{self, Print, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};

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
            MapTooLarge => write!(
                f,
                "Board too big, maximum width and height for TUI : {}",
                u16::MAX
            ),
        }
    }
}

impl Error for TuiError {}

/// Base command-line interface.
/// The whole scene is reprinted each step and the input isn't real-time.
pub struct Tui {
    original_cols: u16,
    original_rows: u16,
}

impl Ui for Tui {
    fn initialize() -> Result<Self, Box<dyn Error>> {
        // TODO: is resize even needed ?
        /*
        let cols = u16::try_from(board.width()).map_err(|_| TuiError::MapTooLarge)?;
        let rows = u16::try_from(board.height()).map_err(|_| TuiError::MapTooLarge)?;
        */
        let res: Result<(u16, u16), io::Error> = try {
            terminal::enable_raw_mode()?;

            let original_cols_rows = terminal::size()?;

            let mut stdout = io::stdout();

            stdout
                // .queue(terminal::SetSize(cols, rows))?;
                .queue(cursor::Hide)?
                // .queue(terminal::Clear(terminal::ClearType::All))?
                .queue(terminal::EnterAlternateScreen)?
                .queue(terminal::SetTitle("Sooban"))?;

            stdout.flush()?;

            original_cols_rows
        };
        let (original_cols, original_rows) = res.map_err(|e| Box::new(TuiError::IO(e)))?;

        Ok(Tui {
            original_cols,
            original_rows,
        })
    }

    fn cleanup(&self) -> Result<(), Box<dyn Error>> {
        let res: Result<(), io::Error> = try {
            // io::stdout().execute(terminal::SetSize(self.original_cols, self.original_rows))?;
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
                Event::Key(KeyEvent {
                    modifiers: KeyModifiers::NONE,
                    code,
                    ..
                }) => match code {
                    KeyCode::Char('q') => break Action::Quit,
                    KeyCode::Char('r') => break Action::Quit,
                    KeyCode::Left => break Action::Movement(Direction::Left),
                    KeyCode::Right => break Action::Movement(Direction::Right),
                    KeyCode::Up => break Action::Movement(Direction::Up),
                    KeyCode::Down => break Action::Movement(Direction::Down),
                    _ => (),
                },
                Event::Key(KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code,
                    ..
                }) => match code {
                    KeyCode::Char('c') => break Action::Quit,
                    _ => (),
                },
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
        let res: Result<(), io::Error> = try {
            let mut stdout = io::stdout();

            for y in 0..40 {
                for x in 0..150 {
                    if (y == 0 || y == 40 - 1) || (x == 0 || x == 150 - 1) {
                        // in this loop we are more efficient by not flushing the buffer.
                        stdout
                            .queue(cursor::MoveTo(x, y))?
                            .queue(style::PrintStyledContent("█".magenta()))?;
                    }
                }
            }
            stdout.flush()?;
        };
        res.map_err(|e| Box::new(TuiError::IO(e)))?;

        Ok(())
    }

    fn won(&self) -> Result<(), Box<dyn Error>> {
        let res: Result<(), io::Error> = try {};
        res.map_err(|e| Box::new(TuiError::IO(e)))?;

        todo!();

        // Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
