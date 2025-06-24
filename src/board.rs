use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

#[cfg(feature = "savestates")]
use bincode::{Decode, Encode};

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "savestates", derive(Encode, Decode))]
#[repr(u8)]
pub enum Slot {
    Empty,
    X,
    O,
    Disabled,
}

impl Slot {
    pub fn to_chr(self) -> char {
        match self {
            Slot::Empty => ' ',
            Slot::X => 'X',
            Slot::O => 'O',
            Slot::Disabled => '_',
        }
    }

    pub fn flip(self) -> Self {
        match self {
            Slot::X => Slot::O,
            Slot::O => Slot::X,
            _ => self,
        }
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_chr())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "savestates", derive(Encode, Decode))]
pub enum State {
    Won,
    Lost,
    Tied,
    Undecided,
}

impl State {
    pub const fn to_u32(self) -> u32 {
        match self {
            State::Undecided => 0,
            State::Won => 1,
            State::Lost => 2,
            State::Tied => 3,
        }
    }

    pub const fn from_u32(bits: u32) -> State {
        match bits {
            0 => State::Undecided,
            1 => State::Won,
            2 => State::Lost,
            3 => State::Tied,

            _ => unreachable!(),
        }
    }

    pub const fn flip(self) -> Self {
        match self {
            State::Won => Self::Lost,
            State::Lost => Self::Won,

            _ => self,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "savestates", derive(Encode, Decode))]
pub struct Board(pub [Slot; 9]);

impl Board {
    pub const fn new() -> Self {
        Board([Slot::Empty; 9])
    }

    pub fn new_with(brd: [Slot; 9]) -> Self {
        Board(brd)
    }

    pub const fn rows(self) -> [[Slot; 3]; 3] {
        // SAFETY: arrays should be contiguous in memory
        unsafe { std::mem::transmute(self) }
    }

    pub const fn transpose(self) -> Self {
        let mut b = self;

        b.0.swap(3, 1);
        b.0.swap(6, 2);
        b.0.swap(5, 7);

        b
    }

    pub const fn columns(self) -> [[Slot; 3]; 3] {
        self.transpose().rows()
    }

    pub const fn diags(self) -> [[Slot; 3]; 2] {
        let b = self.0;

        [[b[0], b[4], b[8]], [b[2], b[4], b[6]]]
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for Board {
    type Output = Slot;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Board {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[cfg(test)]
mod test {
    use super::{Board, Slot};

    #[test]
    fn does_rows_cause_ub() {
        let game = Board::new();

        game.rows()[0][2] = Slot::X;
    }
}
