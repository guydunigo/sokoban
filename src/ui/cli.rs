//! Base command-line interface.
use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

use super::{Action, Board, BoardElem, CellKind, Direction, MovableItem, Ui};

const SYMBOL_VOID: char = ' ';
const SYMBOL_FLOOR: char = '.';
const SYMBOL_WALL: char = '#';
const SYMBOL_TARGET: char = 'X';
const SYMBOL_PLAYER: char = 'P';
const SYMBOL_PLAYER_ON_TARGET: char = 'R';
const SYMBOL_CRATE: char = 'O';
const SYMBOL_PLACED_CRATE: char = '8';

#[derive(Debug)]
pub enum CliError {
    IO(io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CliError::*;
        match self {
            IO(other) => write!(f, "IO error when reading player input: {}", other),
        }
    }
}

impl Error for CliError {}

/// Base command-line interface.
/// The whole scene is reprinted each step and the input isn't real-time.
pub struct Cli;

impl Cli {
    pub fn new() -> Self {
        // TODO: add symbol description based on constants.
        println!("Welcome in my Sokoban.\nPush the crates around until all of them are placed on a target.\nEach turn, you must enter a command followed by 'enter': left (l), right (r), up (u), down (d), reset (re) or quit (qu).\n\nSymbols:\n- {} : floor\n- {} : wall\n- {} : target\n- {} : player\n- {} : player on a target (nothing particular, just to know there's a terget under)\n- {} : crate\n- {} : crate placed on a target (in the end, all crate should look like that).\n", SYMBOL_FLOOR, SYMBOL_WALL, SYMBOL_TARGET, SYMBOL_PLAYER, SYMBOL_PLAYER_ON_TARGET, SYMBOL_CRATE, SYMBOL_PLACED_CRATE);

        Cli
    }
}

impl Ui for Cli {
    fn get_input(&mut self) -> Result<Action, Box<dyn Error>> {
        loop {
            print!("> ");

            // We need to flush, otherwise the call to stdin locks the program before
            // the buffer is actually printed. (try it if you don't believe it)
            io::stdout()
                .flush()
                .map_err(|e| Box::new(CliError::IO(e)))?;

            let mut buffer = String::new();
            match io::stdin().read_line(&mut buffer) {
                Ok(0) => break Ok(Action::Quit),
                Ok(_) => match &buffer.trim().to_lowercase()[..] {
                    "l" | "left" => break Ok(Action::Movement(Direction::Left)),
                    "r" | "right" => break Ok(Action::Movement(Direction::Right)),
                    "u" | "up" => break Ok(Action::Movement(Direction::Up)),
                    "d" | "down" => break Ok(Action::Movement(Direction::Down)),
                    "re" | "reset" => break Ok(Action::ResetLevel),
                    "qu" | "quit" => break Ok(Action::Quit),
                    _ => println!("Unknown command `{}`, please try again:", buffer.trim()),
                },
                Err(e) => break Err(Box::new(CliError::IO(e))),
            }
        }
    }

    fn display(
        &mut self,
        board: &Board,
        _last_move_result: Option<Option<(isize, isize)>>,
    ) -> Result<(), Box<dyn Error>> {
        let width = board.width();
        let height = board.height();
        for j in 0..height {
            for i in 0..width {
                use CellKind::*;
                use MovableItem::*;

                print!(
                    "{}",
                    match board.get(i as isize, j as isize) {
                        BoardElem(_, Void) => SYMBOL_VOID,
                        BoardElem(_, Wall) => SYMBOL_WALL,
                        BoardElem(None, Floor) => SYMBOL_FLOOR,
                        BoardElem(None, Target) => SYMBOL_TARGET,
                        BoardElem(Some(Player), Floor) => SYMBOL_PLAYER,
                        BoardElem(Some(Crate(_)), Floor) => SYMBOL_CRATE,
                        BoardElem(Some(Player), Target) => SYMBOL_PLAYER_ON_TARGET,
                        BoardElem(Some(Crate(_)), Target) => SYMBOL_PLACED_CRATE,
                    }
                )
            }
            println!();
        }

        Ok(())
    }

    fn won(&mut self) -> Result<(), Box<dyn Error>> {
        println!("+----------+");
        println!("| You won! |");
        println!("+----------+");
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
