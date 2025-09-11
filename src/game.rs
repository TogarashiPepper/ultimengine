use std::array;

#[cfg(feature = "savestates")]
use bincode::{Decode, Encode};
use rand::{SeedableRng, seq::IndexedRandom};

use crate::{
    bitboard::BitBoard,
    board::{Slot, State},
    counting::possible_to_win,
    moves::{Move, is_legal, legal_moves},
};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "savestates", derive(Encode, Decode))]
pub struct Game {
    pub boards: [BitBoard; 9],
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
            boards: [BitBoard::new(); 9],
            active: 9,
            state: State::Undecided,
            last_move: None,
        }
    }

    pub fn random(times: u8) -> Game {
        let mut g = Game::new();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let mut side = Slot::X;

        for _ in 0..times {
            let lgms = legal_moves(&g);

            if lgms.is_empty() {
                break;
            }

            g.make_move(*lgms.choose(&mut rng).unwrap(), side).unwrap();

            side = side.flip();
        }

        g
    }

    pub fn random_seedless(times: u8) -> Game {
        let mut g = Game::new();
        let mut rng = rand::rng();
        let mut side = Slot::X;

        for _ in 0..times {
            let lgms = legal_moves(&g);

            if lgms.is_empty() {
                break;
            }

            g.make_move(*lgms.choose(&mut rng).unwrap(), side).unwrap();

            side = side.flip();
        }

        g
    }

    pub fn _test() -> Self {
        use Slot::{Empty as E, O, X};

        let mut g = Self::new();

        // Tie board
        g.boards[0] = BitBoard::new_with([X, O, X, X, O, O, O, X, X]);
        g.boards[0].set_state(State::Tied);
        // Win Board
        g.boards[1] = BitBoard::new_with([X, X, X, E, E, E, E, E, E]);
        g.boards[1].set_state(State::Won);
        // Loss Board
        g.boards[2] = BitBoard::new_with([O, O, O, E, E, E, E, E, E]);
        g.boards[2].set_state(State::Lost);

        g
    }

    pub fn flip(&self) -> Self {
        let mut new = self.clone();

        for board in &mut new.boards {
            board.flip();
        }

        new.state = new.state.flip();

        new
    }

    pub fn shrink(&self) -> BitBoard {
        let arr = array::from_fn(|idx| match self.boards[idx].state() {
            State::Won => Slot::X,
            State::Lost => Slot::O,
            State::Tied => Slot::Disabled,
            State::Undecided => Slot::Empty,
        });

        BitBoard::new_with(arr)
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

        brd.0 |= 1
            << (mv.index
                + match side {
                    Slot::X => 0,
                    Slot::O => 9,
                    Slot::Empty => 18,
                    Slot::Disabled => unreachable!(),
                });

        let idx = 1 << (18 + mv.index);
        brd.0 &= !idx;

        if brd.won_by_x() {
            self.boards[mv.game].set_state(State::Won);
        } else if brd.won_by_o() {
            self.boards[mv.game].set_state(State::Lost);
        } else if !possible_to_win(*brd) {
            self.boards[mv.game].set_state(State::Tied);
        }

        let shrunken = self.shrink();
        if shrunken.won_by_x() {
            self.state = State::Won;
        } else if shrunken.won_by_o() {
            self.state = State::Lost;
        } else if !possible_to_win(shrunken) {
            self.state = State::Tied;
        }

        if self.boards[mv.index].state() != State::Undecided {
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

                        *byte = self.state_to_col(self.boards[idx].state(), idx);
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
                            *byte = self.state_to_col(self.boards[gm].state(), gm);
                        }
                    }
                    _ => {
                        let idx = (*byte - b'a') as usize;
                        *byte = self.boards[idx].to_arr()[idxs[idx]].to_chr() as u8;

                        idxs[idx] += 1;
                    }
                }
            }
        }

        String::from_utf8(res).unwrap()
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
