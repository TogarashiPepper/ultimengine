use std::fmt::Display;

#[cfg(feature = "savestates")]
use bincode::{Decode, Encode};

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
