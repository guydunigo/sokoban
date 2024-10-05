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
    pub fn to_coords(self, i: u32, j: u32) -> (u32, u32) {
        use Direction::*;

        let (mut res_i, mut res_j) = (i, j);

        // Can't go below 0 :
        match self {
            Left => res_i = res_i.saturating_sub(1),
            Right => res_i = res_i.saturating_add(1),
            Up => res_j = res_j.saturating_sub(1),
            Down => res_j = res_j.saturating_add(1),
        };

        (res_i, res_j)
    }
}

/// Crate which can be pushed unless there is an *uncrossable* cell (see [`CellKind::is_crossable`]) or another crate in the way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Crate {
    i: u32,
    j: u32,
}

impl Crate {
    pub fn new(i: u32, j: u32) -> Self {
        Crate { i, j }
    }

    pub fn pos(&self) -> (u32, u32) {
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
        matches!(board.map().get(self.i, self.j), CellKind::Target)
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
