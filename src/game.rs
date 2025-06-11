use std::array;

#[cfg(feature = "savestates")]
use bincode::{Decode, Encode};

use crate::{
    board::{Board, Slot, State},
    counting::{possible_to_win, won_for},
    moves::{is_legal, Move},
};

#[derive(Clone)]
#[cfg_attr(feature = "savestates", derive(Encode, Decode))]
pub struct Game {
    pub boards: [Board; 9],
    pub states: [State; 9],
    /// Indicates active board, 0-8 is the idx, 9 means any board is free
    pub active: usize,
    pub state: State,
    pub last_move: Option<Move>,
}

/// Template for board, a-i represents which board
/// x means insert fmt char, y means insert reset char
const TEMPLATE: &[u8] = b"
  \x1b[0;3nm\x1b[0;3Ama\x1b[0;3nm | \x1b[0;3Ama\x1b[0;3nm | \x1b[0;3Ama\x1b[0;3nm\x1b[0m   |   \x1b[0;3om\x1b[0;3Bmb\x1b[0;3om | \x1b[0;3Bmb\x1b[0;3om | \x1b[0;3Bmb\x1b[0;3om\x1b[0m   |   \x1b[0;3pm\x1b[0;3Cmc\x1b[0;3pm | \x1b[0;3Cmc\x1b[0;3pm | \x1b[0;3Cmc\x1b[0;3pm\x1b[0m   
 \x1b[0;3nm-----------\x1b[0m  |  \x1b[0;3om-----------\x1b[0m  |  \x1b[0;3pm-----------\x1b[0m  
  \x1b[0;3nm\x1b[0;3Ama\x1b[0;3nm | \x1b[0;3Ama\x1b[0;3nm | \x1b[0;3Ama\x1b[0;3nm\x1b[0m   |   \x1b[0;3om\x1b[0;3Bmb\x1b[0;3om | \x1b[0;3Bmb\x1b[0;3om | \x1b[0;3Bmb\x1b[0;3om\x1b[0m   |   \x1b[0;3pm\x1b[0;3Cmc\x1b[0;3pm | \x1b[0;3Cmc\x1b[0;3pm | \x1b[0;3Cmc\x1b[0;3pm\x1b[0m   
 \x1b[0;3nm-----------\x1b[0m  |  \x1b[0;3om-----------\x1b[0m  |  \x1b[0;3pm-----------\x1b[0m  
  \x1b[0;3nm\x1b[0;3Ama\x1b[0;3nm | \x1b[0;3Ama\x1b[0;3nm | \x1b[0;3Ama\x1b[0;3nm\x1b[0m   |   \x1b[0;3om\x1b[0;3Bmb\x1b[0;3om | \x1b[0;3Bmb\x1b[0;3om | \x1b[0;3Bmb\x1b[0;3om\x1b[0m   |   \x1b[0;3pm\x1b[0;3Cmc\x1b[0;3pm | \x1b[0;3Cmc\x1b[0;3pm | \x1b[0;3Cmc\x1b[0;3pm\x1b[0m   
              |               |               
---------------------------------------------
              |               |
  \x1b[0;3qm\x1b[0;3Dmd\x1b[0;3qm | \x1b[0;3Dmd\x1b[0;3qm | \x1b[0;3Dmd\x1b[0;3qm\x1b[0m   |   \x1b[0;3rm\x1b[0;3Eme\x1b[0;3rm | \x1b[0;3Eme\x1b[0;3rm | \x1b[0;3Eme\x1b[0;3rm\x1b[0m   |   \x1b[0;3sm\x1b[0;3Fmf\x1b[0;3sm | \x1b[0;3Fmf\x1b[0;3sm | \x1b[0;3Fmf\x1b[0;3sm\x1b[0m   
 \x1b[0;3qm-----------\x1b[0m  |  \x1b[0;3rm-----------\x1b[0m  |  \x1b[0;3sm-----------\x1b[0m  
  \x1b[0;3qm\x1b[0;3Dmd\x1b[0;3qm | \x1b[0;3Dmd\x1b[0;3qm | \x1b[0;3Dmd\x1b[0;3qm\x1b[0m   |   \x1b[0;3rm\x1b[0;3Eme\x1b[0;3rm | \x1b[0;3Eme\x1b[0;3rm | \x1b[0;3Eme\x1b[0;3rm\x1b[0m   |   \x1b[0;3sm\x1b[0;3Fmf\x1b[0;3sm | \x1b[0;3Fmf\x1b[0;3sm | \x1b[0;3Fmf\x1b[0;3sm\x1b[0m   
 \x1b[0;3qm-----------\x1b[0m  |  \x1b[0;3rm-----------\x1b[0m  |  \x1b[0;3sm-----------\x1b[0m  
  \x1b[0;3qm\x1b[0;3Dmd\x1b[0;3qm | \x1b[0;3Dmd\x1b[0;3qm | \x1b[0;3Dmd\x1b[0;3qm\x1b[0m   |   \x1b[0;3rm\x1b[0;3Eme\x1b[0;3rm | \x1b[0;3Eme\x1b[0;3rm | \x1b[0;3Eme\x1b[0;3rm\x1b[0m   |   \x1b[0;3sm\x1b[0;3Fmf\x1b[0;3sm | \x1b[0;3Fmf\x1b[0;3sm | \x1b[0;3Fmf\x1b[0;3sm\x1b[0m   
              |               |               
---------------------------------------------
              |               |
  \x1b[0;3tm\x1b[0;3Gmg\x1b[0;3tm | \x1b[0;3Gmg\x1b[0;3tm | \x1b[0;3Gmg\x1b[0;3tm\x1b[0m   |   \x1b[0;3um\x1b[0;3Hmh\x1b[0;3um | \x1b[0;3Hmh\x1b[0;3um | \x1b[0;3Hmh\x1b[0;3um\x1b[0m   |   \x1b[0;3vm\x1b[0;3Imi\x1b[0;3vm | \x1b[0;3Imi\x1b[0;3vm | \x1b[0;3Imi\x1b[0;3vm\x1b[0m   
 \x1b[0;3tm-----------\x1b[0m  |  \x1b[0;3um-----------\x1b[0m  |  \x1b[0;3vm-----------\x1b[0m  
  \x1b[0;3tm\x1b[0;3Gmg\x1b[0;3tm | \x1b[0;3Gmg\x1b[0;3tm | \x1b[0;3Gmg\x1b[0;3tm\x1b[0m   |   \x1b[0;3um\x1b[0;3Hmh\x1b[0;3um | \x1b[0;3Hmh\x1b[0;3um | \x1b[0;3Hmh\x1b[0;3um\x1b[0m   |   \x1b[0;3vm\x1b[0;3Imi\x1b[0;3vm | \x1b[0;3Imi\x1b[0;3vm | \x1b[0;3Imi\x1b[0;3vm\x1b[0m   
 \x1b[0;3tm-----------\x1b[0m  |  \x1b[0;3um-----------\x1b[0m  |  \x1b[0;3vm-----------\x1b[0m  
  \x1b[0;3tm\x1b[0;3Gmg\x1b[0;3tm | \x1b[0;3Gmg\x1b[0;3tm | \x1b[0;3Gmg\x1b[0;3tm\x1b[0m   |   \x1b[0;3um\x1b[0;3Hmh\x1b[0;3um | \x1b[0;3Hmh\x1b[0;3um | \x1b[0;3Hmh\x1b[0;3um\x1b[0m   |   \x1b[0;3vm\x1b[0;3Imi\x1b[0;3vm | \x1b[0;3Imi\x1b[0;3vm | \x1b[0;3Imi\x1b[0;3vm\x1b[0m   
";

impl Game {
    pub fn new() -> Self {
        Game {
            boards: [Board::new(); 9],
            states: [State::Undecided; 9],
            active: 9,
            state: State::Undecided,
            last_move: None,
        }
    }

    pub fn _test() -> Self {
        use Slot::{Empty as E, O, X};

        let mut g = Self::new();

        // Tie board
        g.boards[0] = Board::new_with([X, O, X, X, O, O, O, X, X]);
        g.states[0] = State::Tied;
        // Win Board
        g.boards[1] = Board::new_with([X, X, X, E, E, E, E, E, E]);
        g.states[1] = State::Won;
        // Loss Board
        g.boards[2] = Board::new_with([O, O, O, E, E, E, E, E, E]);
        g.states[2] = State::Lost;

        g
    }

    pub fn shrink(&self) -> Board {
        let arr = array::from_fn(|idx| match self.states[idx] {
            State::Won => Slot::X,
            State::Lost => Slot::O,
            State::Tied => Slot::Disabled,
            State::Undecided => Slot::Empty,
        });

        Board::new_with(arr)
    }

    pub fn sim_move(&self, mv: Move, side: Slot) -> Result<Game, &'static str> {
        let mut new = self.clone();

        new.make_move(mv, side)?;

        Ok(new)
    }

    pub fn make_move(&mut self, mv: Move, side: Slot) -> Result<(), &'static str> {
        is_legal(self, mv)?;

        if self.active == 9 {
            self.active = mv.game;
        }

        let brd = &mut self.boards[mv.game];

        brd[mv.index] = side;
        if won_for(*brd, Slot::X) {
            self.states[mv.game] = State::Won;
        } else if won_for(*brd, Slot::O) {
            self.states[mv.game] = State::Lost;
        } else if !possible_to_win(*brd) {
            self.states[mv.game] = State::Tied;
        }

        let shrunken = self.shrink();
        if won_for(shrunken, Slot::X) {
            self.state = State::Won;
        } else if won_for(shrunken, Slot::O) {
            self.state = State::Lost;
        } else if !possible_to_win(shrunken) {
            self.state = State::Tied;
        }

        if self.states[mv.index] != State::Undecided {
            self.active = 9;
        } else {
            self.active = mv.index;
        }

        self.last_move = Some(mv);

        Ok(())
    }

    fn state_to_col(&self, state: State, idx: usize) -> u8 {
        match state {
            State::Won => b'1',
            State::Lost => b'2',
            State::Tied => b'3',
            State::Undecided if idx == self.active || self.active == 9 => b'5',
            _ => b'7',
        }
    }

    pub fn print(&self) -> String {
        let mut res = TEMPLATE.to_vec();
        let mut idxs = [0; 9];

        for byte in res.iter_mut() {
            if byte.is_ascii_alphabetic() {
                match byte {
                    b'n'..=b'v' => {
                        let idx = (*byte - b'n') as usize;

                        *byte = self.state_to_col(self.states[idx], idx);
                    }
                    b'm' => continue,
                    b'A'..=b'Z' => {
                        let gm = (*byte - b'A') as usize;
                        let idx = idxs[gm];

                        if self.last_move
                            == Some(Move {
                                game: gm,
                                index: idx,
                            })
                        {
                            *byte = b'4';
                        } else {
                            *byte = self.state_to_col(self.states[gm], gm);
                        }
                    }
                    _ => {
                        let idx = (*byte - b'a') as usize;
                        *byte = self.boards[idx][idxs[idx]].to_chr() as u8;

                        idxs[idx] += 1;
                    }
                }
            }
        }

        String::from_utf8(res).unwrap()
    }
}
