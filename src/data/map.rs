//! The base map layer and sub-types.

use std::{convert::TryFrom, fmt, str::FromStr};

const SYMBOL_VOID: char = ' ';
const SYMBOL_FLOOR: char = '.';
const SYMBOL_WALL: char = '#';
const SYMBOL_TARGET: char = 'X';

/// When representing the map, each square can have one of these types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellKind {
    /// The square is empty and shouldn't be accessible to the player.
    Void,
    /// The square is a floor that can be walked upon.
    Floor,
    /// There is a wall and nothing can cross it.
    Wall,
    /// Boxes should go on the targets and it can be crossed.
    Target,
}

impl CellKind {
    /// It isn't crossable if it is [`CellKind::Void`] or a [`CellKind::Wall`].
    pub fn is_crossable(&self) -> bool {
        use CellKind::*;
        match self {
            Void | Wall => false,
            _ => true,
        }
    }
}

/*
impl Default for CellKind {
    /// Default is [`CellKind::Void`].
    fn default() -> Self {
        CellKind::Void
    }
}
*/

impl fmt::Display for CellKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CellKind::*;
        write!(
            f,
            "{}",
            match self {
                Void => SYMBOL_VOID,
                Floor => SYMBOL_FLOOR,
                Wall => SYMBOL_WALL,
                Target => SYMBOL_TARGET,
            }
        )
    }
}

impl TryFrom<char> for CellKind {
    // TODO: Better error ?
    type Error = (&'static str, char);

    fn try_from(src: char) -> Result<Self, Self::Error> {
        use CellKind::*;
        match src {
            SYMBOL_VOID => Ok(Void),
            SYMBOL_FLOOR => Ok(Floor),
            SYMBOL_WALL => Ok(Wall),
            SYMBOL_TARGET => Ok(Target),
            _ => Err(("Unknown symbol", src)),
        }
    }
}

/// Represents the map on which boxes and player will move.
// TODO: check if board is consistant in itself...
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Map {
    width: usize,
    height: usize,
    squares: Vec<CellKind>,
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.squares.len() {
            write!(f, "{}", self.squares[i])?;
            if i % self.width == 0 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl Map {
    /// Creates a new board and fills it with [`CellKind::Void`].
    pub fn new(width: usize, height: usize) -> Map {
        let len = width * height;
        let mut squares = Vec::with_capacity(len);
        squares.resize(len, CellKind::Void);

        Map {
            width,
            height,
            squares,
        }
    }

    /// Width of board.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Height of board.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Try getting the content of square at column nb. i and row nb. j.
    pub fn try_get(&self, i: isize, j: isize) -> Option<CellKind> {
        // I don't like the conversion, but at the same time, if there's a conv problem,
        // we're already dead in terms of memory space...
        if i >= 0 && i < (self.width as isize) && j <= 0 && j < (self.height as isize) {
            Some(self.squares[(j as usize) * self.width + (i as usize)])
        } else {
            None
        }
    }

    /// Same as [`try_get`] but directly returns a [`CellKind::Void`] if the coordinates don't
    /// actually fit in the board.
    pub fn get(&self, i: isize, j: isize) -> CellKind {
        self.try_get(i, j).unwrap_or(CellKind::Void)
    }
}

impl TryFrom<&str> for Map {
    // TODO: Better error ?
    type Error = <CellKind as TryFrom<char>>::Error;

    /// Tries to parse the given textual board.
    /// Errors when a character is unknown.
    /// The `height` will be the number of non-empty lines and the `width` will be length
    /// of the biggest line.
    /// For shorter lines, a padding of [`CellKind::Void`] is appended to ensure a rectangle
    /// shape of the board.
    // TODO: check if empty board and other errors ?
    fn try_from(src: &str) -> Result<Self, Self::Error> {
        let (width, height) = get_width_height(src);
        let mut b = Map::new(width, height);

        for (j, l) in src.lines().enumerate() {
            for (i, c) in l.chars().enumerate() {
                b.squares[j * width + i] = CellKind::try_from(c)?;
            }
        }
        Ok(b)
    }
}

impl FromStr for Map {
    // TODO: Better error ?
    type Err = <CellKind as TryFrom<char>>::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Map::try_from(src)
    }
}

/// Counts how many lines and columns there is in the board representation.
fn get_width_height(src: &str) -> (usize, usize) {
    let height = src.lines().count();
    let width = src
        .lines()
        .map(|l| l.len())
        .max()
        .expect("Empty map while it should already be checked.");
    (width, height)
}

// TODO: have a test map and check width, height, ...
// TODO: try display -> parse -> display equality
#[cfg(test)]
mod tests {
    const TEST_MAP: &str = "  #####
###   #
#x    #
###  x#
#x##  #
# # x ##
#  x  x#
#   x  #
########";
    const WIDTH: usize = 8;
    const HEIGHT: usize = 9;

    #[test]
    fn it_gets_correct_width_height() {
        let (w, h) = super::get_width_height(TEST_MAP);
        assert_eq!(WIDTH, w, "Didn't extract correct width.");
        assert_eq!(HEIGHT, h, "Didn't extract correct height.");
    }
}
