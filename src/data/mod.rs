//! The data-structures describing the state the game, and how each element interract with each
//! others.

use std::{error::Error, fmt, str::FromStr};

mod map;
pub use map::{CellKind, Map};
mod movable;
pub use movable::{Crate, Direction};

/// Item maybe found on top of a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovableItem<'a> {
    Player,
    Crate(&'a Crate),
}

pub struct BoardElem<'a>(Option<MovableItem<'a>>, CellKind);

/*
// TODO: Display shouldn't be done here...

const SYMBOL_PLAYER: char = 'P';
const SYMBOL_CRATE: char = '#';
const SYMBOL_PLACED_CRATE: char = 'x';

impl fmt::Display for BoardElem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CellKind::*;
        use MovableItem::*;
        match self {
            BoardElem(None, c) => write!(f, "{}", c),
            BoardElem(Some(Player), _) => write!(f, "{}", SYMBOL_PLAYER),
            BoardElem(Some(Crate), Floor) => write!(f, "{}", SYMBOL_CRATE),
            BoardElem(Some(Crate), Target) => write!(f, "{}", SYMBOL_PLACED_CRATE),
        }
    }
}
*/

/// The [`Board`] contains the [`Map`], the items ([crates](`Crate`) and the [player](`Player`)) on
/// top.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    map: Map,
    player: (isize, isize),
    crates: Vec<Crate>,
}

impl Board {
    fn map(&self) -> &Map {
        &self.map
    }

    fn try_get<'a>(&'a self, i: isize, j: isize) -> Option<BoardElem<'a>> {
        if let Some(c) = self.map.try_get(i, j) {
            if self.player == (i, j) {
                Some(BoardElem(Some(MovableItem::Player), c))
            } else if let Some(b) = self.crates.iter().find(|c| c.pos() == (i, j)) {
                Some(BoardElem(Some(MovableItem::Crate(&b)), c))
            } else {
                Some(BoardElem(None, c))
            }
        } else {
            None
        }
    }

    fn get<'a>(&'a self, i: isize, j: isize) -> BoardElem<'a> {
        let c = self.map.get(i, j);

        if self.player == (i, j) {
            BoardElem(Some(MovableItem::Player), c)
        } else if let Some(b) = self.crates.iter().find(|c| c.pos() == (i, j)) {
            BoardElem(Some(MovableItem::Crate(b)), c)
        } else {
            BoardElem(None, c)
        }
    }

    /// The player can move unless there is an *uncrossable* cell (see [`CellKind::is_crossable`]) or an unmovable crate in the way.
    /// If there is a crate which can be moved in the same direction, it will (see
    /// [`Crate::can_move`]).
    ///
    /// Returns:
    /// - `Some(true)` if it can move by pushing a crate,
    /// - `Some(false)` if it can move without pushing a crate,
    /// - `None` if it can't move at all.
    pub fn can_player_move(self: &Board, dir: Direction) -> Option<bool> {
        let (i, j) = dir.to_coords(self.player.0, self.player.1);
        match self.get(i, j) {
            BoardElem(Some(MovableItem::Crate(c)), _) => {
                if c.can_move(self, dir) {
                    Some(true)
                } else {
                    None
                }
            }
            BoardElem(Some(MovableItem::Player), _) => unreachable!(), // at least for now...
            BoardElem(None, c) => {
                if c.is_crossable() {
                    Some(false)
                } else {
                    None
                }
            }
        }
    }

    /// Actually moves if it can move and returns `true`, or `false` if it couldn't move.
    /// Returns:
    /// - `Some(Some(&Crate))` if it can move by pushing a crate,
    /// - `Some(None)` if it can move without pushing a crate,
    /// - `None` if it can't move at all.
    pub fn do_move_player<'a>(&'a mut self, dir: Direction) -> Option<Option<&'a Crate>> {
        if let Some(is_crate) = self.can_player_move(dir) {
            let (i, j) = dir.to_coords(self.player.0, self.player.1);

            // If there's a crate to be pushed, move it first:
            let c_opt: Option<&'a Crate> = if is_crate {
                let c = self.crates.iter_mut().find(|c| c.pos() == (i, j)).expect(
                    "It was annouced that the player would push a crate, but there isn't any.",
                );
                c.do_move(dir);

                /*
                // TODO: no need to re-check here ?
                if c.can_move(self, dir) {
                    c.do_move(dir);
                } else {
                    unreachable!("Crate can't move but it should already have been checked.");
                }
                */

                Some(c)
            } else {
                None
            };

            self.player = (i, j);
            Some(c_opt)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LevelParseError {
    CantParseMap(<Map as FromStr>::Err),
}

impl fmt::Display for LevelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // TODO: not debug
            LevelParseError::CantParseMap(err) => write!(f, "Can't parse map string: {:?}", err),
        }
    }
}

impl Error for LevelParseError {}

impl FromStr for Board {
    // TODO: Better error ?
    type Err = LevelParseError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        src.lines()
    }
}
