//! Objects which can be moved on the board.

use super::{Board, BoardElem, CellKind};

/// Direction a [`Movable`] can be moved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn to_coords(&self, i: isize, j: isize) -> (isize, isize) {
        use Direction::*;

        let delta = match self {
            Left => (-1, 0),
            Right => (1, 0),
            Up => (0, -1),
            Down => (0, 1),
        };

        (i + delta.0, j + delta.1)
    }
}

/// Crate which can be pushed unless there is an *uncrossable* cell (see [`CellKind::is_crossable`]) or another crate in the way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Crate {
    i: isize,
    j: isize,
}

impl Crate {
    pub fn new(i: isize, j: isize) -> Self {
        Crate { i, j }
    }

    pub fn pos(&self) -> (isize, isize) {
        (self.i, self.j)
    }

    /// Can move if there's nothing on the cell and if the cell is
    /// [crossable](`CellKind::is_crossable`).
    pub fn can_move(&self, board: &Board, dir: Direction) -> bool {
        let (i, j) = dir.to_coords(self.i, self.j);
        match board.get(i, j) {
            BoardElem(Some(_), _) => false,
            BoardElem(None, c) => c.is_crossable(),
        }
    }

    /// Actually change the coordinates of the crates, but you'd better ensure it can be moved
    /// (using [`Crate::can_move`]).
    pub fn do_move(&mut self, dir: Direction) {
        let (i, j) = dir.to_coords(self.i, self.j);
        self.i = i;
        self.j = j;
    }

    /// If it is on a [`CellKind::Target`].
    pub fn is_placed(&self, board: &Board) -> bool {
        if let CellKind::Target = board.map().get(self.i as isize, self.j as isize) {
            true
        } else {
            false
        }
    }
}

/*
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
*/
