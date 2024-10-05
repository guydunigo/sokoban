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
    // TODO: test ?
    pub fn is_crossable(&self) -> bool {
        use CellKind::*;
        !matches!(self, Void | Wall)
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
    width: u32,
    height: u32,
    squares: Vec<CellKind>,
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.squares.len() {
            if i > 0
                && i % usize::try_from(self.width).expect("Map width should fit in a usize.") == 0
            {
                writeln!(f)?;
            }
            write!(f, "{}", self.squares[i])?;
        }
        Ok(())
    }
}

impl Map {
    /// Creates a new board and fills it with [`CellKind::Void`].
    pub fn new(width: u32, height: u32) -> Map {
        let len =
            usize::try_from(width * height).expect("Number of map squares should fit in usize.");
        let mut squares = Vec::with_capacity(len);
        squares.resize(len, CellKind::Void);

        Map {
            width,
            height,
            squares,
        }
    }

    /// Width of board.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of board.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Try getting the content of square at column nb. i and row nb. j.
    pub fn try_get(&self, i: u32, j: u32) -> Option<CellKind> {
        if i < self.width && j < self.height {
            Some(
                self.squares
                    [usize::try_from(j * self.width + i).expect("Square id should fit in usize.")],
            )
        } else {
            None
        }
    }

    /// Same as [`try_get`] but directly returns a [`CellKind::Void`] if the coordinates don't
    /// actually fit in the board.
    pub fn get(&self, i: u32, j: u32) -> CellKind {
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
                b.squares[j * usize::try_from(width).expect("Width should fit in usize") + i] =
                    CellKind::try_from(c)?;
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
fn get_width_height(src: &str) -> (u32, u32) {
    let height = src.lines().count();
    let width = src
        .lines()
        .map(|l| l.len())
        .max()
        .expect("Empty map while it should already be checked.");

    (
        width.try_into().expect("Width should fit in u32"),
        height.try_into().expect("Height should fit in u32"),
    )
}

// TODO: have a test map and check width, height, ...
// TODO: try display -> parse -> display equality
#[cfg(test)]
mod tests {
    use super::{CellKind::*, Map};

    const TEST_MAP_STR: &str = "  #####
###...#
#X....#
###..X#
#X##..#
#.#.X.##
#..X..X#
#...X..#
########";

    const WIDTH: u32 = 8;
    const HEIGHT: u32 = 9;

    fn test_map() -> Map {
        Map {
            width: WIDTH,
            height: HEIGHT,
            squares: vec![
                Void, Void, Wall, Wall, Wall, Wall, Wall, Void, Wall, Wall, Wall, Floor, Floor,
                Floor, Wall, Void, Wall, Target, Floor, Floor, Floor, Floor, Wall, Void, Wall,
                Wall, Wall, Floor, Floor, Target, Wall, Void, Wall, Target, Wall, Wall, Floor,
                Floor, Wall, Void, Wall, Floor, Wall, Floor, Target, Floor, Wall, Wall, Wall,
                Floor, Floor, Target, Floor, Floor, Target, Wall, Wall, Floor, Floor, Floor,
                Target, Floor, Floor, Wall, Wall, Wall, Wall, Wall, Wall, Wall, Wall, Wall,
            ],
        }
    }

    #[test]
    fn it_gets_correct_width_height() {
        let (w, h) = super::get_width_height(TEST_MAP_STR);
        assert_eq!(WIDTH, w, "Didn't extract correct width.");
        assert_eq!(HEIGHT, h, "Didn't extract correct height.");
    }

    #[test]
    fn it_correctly_parses_map() {
        let reference = test_map();
        let parsed: Map = TEST_MAP_STR.parse().unwrap();
        assert_eq!(reference, parsed);
    }

    #[test]
    fn it_correctly_parses_and_displays_map() {
        // In case of trailing space, but this does overcomplicate things (more error prone...).
        let displayed_map = format!("{}", TEST_MAP_STR.parse::<Map>().unwrap())
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("\n");
        let orig_test_map_str = TEST_MAP_STR
            .lines()
            .map(|l| l.trim())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(displayed_map, orig_test_map_str,);
    }

    #[test]
    fn it_gets_a_cell_from_ref_map() {
        let map = test_map();

        assert_eq!(map.try_get(0, 0), Some(Void));
        assert_eq!(map.get(0, 0), Void);

        assert_eq!(map.try_get(0, 1), Some(Wall));
        assert_eq!(map.get(0, 1), Wall);

        assert_eq!(map.try_get(3, 1), Some(Floor));
        assert_eq!(map.get(3, 1), Floor);

        assert_eq!(map.try_get(1, 2), Some(Target));
        assert_eq!(map.get(1, 2), Target);

        // Outside range:
        assert_eq!(map.try_get(WIDTH + 10, 1), None);
        assert_eq!(map.get(WIDTH + 10, 1), Void);
    }

    #[test]
    fn it_gets_a_cell_from_parsed_map() {
        let map: Map = TEST_MAP_STR.parse().unwrap();

        assert_eq!(map.try_get(0, 0), Some(Void));
        assert_eq!(map.get(0, 0), Void);

        assert_eq!(map.try_get(0, 1), Some(Wall));
        assert_eq!(map.get(0, 1), Wall);

        assert_eq!(map.try_get(3, 1), Some(Floor));
        assert_eq!(map.get(3, 1), Floor);

        assert_eq!(map.try_get(1, 2), Some(Target));
        assert_eq!(map.get(1, 2), Target);

        // Outside range:
        assert_eq!(map.try_get(WIDTH + 10, 1), None);
        assert_eq!(map.get(WIDTH + 10, 1), Void);
    }
}
