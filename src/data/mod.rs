//! The data-structures describing the state the game, and how each element interract with each
//! others.

use std::{error::Error, fmt, str::FromStr};

mod map;
pub use map::{CellKind, Map};
mod movable;
pub use movable::{Crate, Direction};

#[cfg(feature = "fyrox")]
use fyrox_core::{
    reflect::{FieldInfo, Reflect},
    visitor::{Visit, VisitResult, Visitor},
};

pub struct BoardElem(pub Option<MovableItem>, pub CellKind);

/// Item maybe found on top of a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovableItem {
    Player,
    Crate(usize),
}

/// The [`Board`] contains the [`Map`], the items ([crates](`Crate`) and the [player](`Player`)) on
/// top.
#[cfg_attr(feature = "fyrox", derive(Default, Reflect))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    map: Map,
    player: (u32, u32),
    crates: Vec<Crate>,
    original_player: (u32, u32),
    original_crates: Vec<Crate>,
}

#[cfg(feature = "fyrox")]
impl Visit for Board {
    fn visit(&mut self, name: &str, visitor: &mut Visitor) -> VisitResult {
        let mut region = visitor.enter_region(name)?;
        self.map.visit("Map", &mut region)?;
        self.player.0.visit("Player0", &mut region)?;
        self.player.1.visit("Player1", &mut region)?;
        self.crates.visit("Crates", &mut region)?;
        self.original_player
            .0
            .visit("OriginalPlayer0", &mut region)?;
        self.original_player
            .1
            .visit("OriginalPlayer1", &mut region)?;
        self.original_crates.visit("OriginalCrates", &mut region)?;
        Ok(())
    }
}

impl Board {
    /*
    pub fn try_get(&self, i: u32, j: u32) -> Option<BoardElem> {
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

    pub fn get(&self, i: u32, j: u32) -> BoardElem {
        let c = self.map.get(i, j);

        if self.player == (i, j) {
            BoardElem(Some(MovableItem::Player), c)
        } else if let Some(i) = self
            .crates
            .iter()
            .enumerate()
            .find(|(_, c)| c.pos() == (i, j))
            .map(|(i, _)| i)
        {
            BoardElem(Some(MovableItem::Crate(i)), c)
        } else {
            BoardElem(None, c)
        }
    }

    pub fn player(&self) -> (u32, u32) {
        self.player
    }

    pub fn crates(&self) -> &[Crate] {
        &self.crates[..]
    }

    /// Actually moves if it can move and returns `true`, or `false` if it couldn't move.
    /// Returns:
    /// - `Some(Some(crate_index))` : the player moved and it moved the crate `board.crates()[crate_index]`
    /// - `Some(None)` : the player moved, but it moved no crate
    /// - `None` if it can't move at all, and the displayed map doesn't need change.
    pub fn do_move_player(&mut self, dir: Direction) -> Option<Option<usize>> {
        let new_player = dir.to_coords(self.player.0, self.player.1);
        let under = self.map.get(new_player.0, new_player.1);

        let (is_crate_blocking, moved_crate) = if let Some(index) = self
            .crates
            .iter()
            .enumerate()
            .find(|(_, c)| c.pos() == (new_player.0, new_player.1))
            .map(|(i, _)| i)
        {
            let new_crate = dir.to_coords(new_player.0, new_player.1);

            if self.map.get(new_crate.0, new_crate.1).is_crossable()
                && self
                    .crates
                    .iter()
                    .find(|c| c.pos() == (new_crate.0, new_crate.1))
                    .is_none()
            {
                self.crates[index].do_move(new_crate.0, new_crate.1);
                (false, Some(index))
            } else {
                (true, None)
            }
        } else {
            (false, None)
        };

        if !under.is_crossable() || is_crate_blocking {
            return None;
        }

        self.player = new_player;

        Some(moved_crate)
    }

    pub fn width(&self) -> u32 {
        self.map.width()
    }

    pub fn height(&self) -> u32 {
        self.map.height()
    }

    pub fn has_won(&self) -> bool {
        self.crates.iter().all(|c| c.is_placed(self))
    }

    pub fn reset(&mut self) {
        self.player = self.original_player;
        self.crates = self.original_crates.clone();
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
                .map(|n| u32::from_str(n).map_err(|_| err()));

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

                let mut c = line.split(',').map(|n| u32::from_str(n).map_err(|_| err()));

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
            original_crates: crates.clone(),
            crates,
            original_player: player,
        })
    }
}
