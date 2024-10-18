//! Objects which can be moved on the board.

use super::{Board, CellKind};

#[cfg(feature = "fyrox")]
use fyrox_core::{
    reflect::{FieldInfo, Reflect},
    visitor::{Visit, VisitResult, Visitor},
};

/// Direction a [`Movable`] can be moved.
#[cfg_attr(feature = "fyrox", derive(Visit, Reflect))]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    #[default]
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
#[cfg_attr(feature = "fyrox", derive(Visit, Reflect, Default))]
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

    /// Actually change the coordinates, make sure they are valid.
    pub fn do_move(&mut self, i: u32, j: u32) {
        self.i = i;
        self.j = j;
    }

    /// If it is on a [`CellKind::Target`].
    pub fn is_placed(&self, board: &Board) -> bool {
        matches!(board.map.get(self.i, self.j), CellKind::Target)
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
