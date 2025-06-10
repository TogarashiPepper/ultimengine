use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

#[derive(Clone, Copy, Debug, PartialEq)]
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
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_chr())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Won,
    Lost,
    Tied,
    Undecided,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Board([Slot; 9]);

impl Board {
    pub const fn new() -> Self {
        Board([Slot::Empty; 9])
    }

    pub fn new_with(brd: [Slot; 9]) -> Self {
        Board(brd)
    }

    pub fn inner(self) -> [Slot; 9] {
        self.0
    }

    pub const fn rows(self) -> [[Slot; 3]; 3] {
        // SAFETY: arrays should be contiguous in memory
        // TODO: miri test
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

    pub fn full(self) -> bool {
        self.0.iter().all(|s| *s != Slot::Empty)
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
