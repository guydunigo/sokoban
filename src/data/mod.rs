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

pub struct BoardElem<'a>(pub Option<MovableItem<'a>>, pub CellKind);

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

    /*
    pub fn try_get(&self, i: isize, j: isize) -> Option<BoardElem> {
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
    */

    pub fn get(&self, i: isize, j: isize) -> BoardElem {
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
    /// - `Some(Some((i,j)))` if it can move by pushing a crate, with (i,j) being the new
    ///   coordinates of the crate,
    /// - `Some(None)` if it can move without pushing a crate,
    /// - `None` if it can't move at all, and the displayed map doesn't need change.
    pub fn do_move_player(&mut self, dir: Direction) -> Option<Option<(isize, isize)>> {
        if let Some(is_crate) = self.can_player_move(dir) {
            let (i, j) = dir.to_coords(self.player.0, self.player.1);

            // If there's a crate to be pushed, move it first:
            let c_opt = if is_crate {
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

                Some(c.pos())
            } else {
                None
            };

            self.player = (i, j);
            Some(c_opt)
        } else {
            None
        }
    }

    pub fn width(&self) -> usize {
        self.map.width()
    }

    pub fn height(&self) -> usize {
        self.map.height()
    }

    pub fn has_won(&self) -> bool {
        self.crates.iter().all(|c| c.is_placed(self))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LevelParseError {
    MissingMap,
    MissingPlayerCoordinates,
    MissingCratesCoordinates,
    CantParseMap(<Map as FromStr>::Err),
    CantParsePlayerCoordinates(String),
    CantParseCrateCoordinates(String),
}

impl fmt::Display for LevelParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use LevelParseError::*;
        match self {
            // TODO: not debug
            MissingMap => write!(f, "Can't find map in file."),
            MissingPlayerCoordinates => write!(f, "Can't find player data in file."),
            MissingCratesCoordinates => write!(f, "Can't find crates data in file."),
            CantParseMap(err) => write!(f, "Can't parse map string: {:?}", err),
            CantParsePlayerCoordinates(err) => {
                write!(f, "Can't parse player coordinates: {:?}", err)
            }
            CantParseCrateCoordinates(err) => write!(f, "Can't parse crate coordinates: {:?}", err),
        }
    }
}

impl Error for LevelParseError {}

impl FromStr for Board {
    type Err = LevelParseError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        // TODO: better format of map will only a map and reading player and crate space from
        // symbols only.
        // TODO: filter necessary?
        let mut blocks = src.split("\n\n").filter(|l| !l.is_empty());

        let map = {
            let map = blocks.next().ok_or(LevelParseError::MissingMap)?;
            Map::from_str(map).map_err(LevelParseError::CantParseMap)?
        };

        let player = {
            let player_line = blocks
                .next()
                .ok_or(LevelParseError::MissingPlayerCoordinates)?;

            let err = || LevelParseError::CantParsePlayerCoordinates(String::from(player_line));

            let mut player = player_line
                .split(',')
                .map(|n| isize::from_str(n).map_err(|_| err()));

            (
                player.next().ok_or_else(err)??,
                player.next().ok_or_else(err)??,
            )
        };

        let crates = {
            let crates_lines = blocks
                .next()
                .ok_or(LevelParseError::MissingCratesCoordinates)?;

            let mut crates = Vec::with_capacity(crates_lines.lines().count());

            for line in crates_lines.lines() {
                // TODO: extract function?
                // TODO: c'est moche...
                let err = || LevelParseError::CantParseCrateCoordinates(String::from(line));

                let mut c = line
                    .split(',')
                    .map(|n| isize::from_str(n).map_err(|_| err()));

                crates.push(Crate::new(
                    c.next().ok_or_else(err)??,
                    c.next().ok_or_else(err)??,
                ));
            }
            crates
        };

        // TODO: ensure all crates have a target?

        Ok(Board {
            map,
            player,
            crates,
        })
    }
}
