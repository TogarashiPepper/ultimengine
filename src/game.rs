use crate::{
    board::{Board, Slot, State},
    counting::won_for, moves::Move,
};

pub struct Game {
    // TODO: AOS vs SOA, must benchmark (prob insignificant w/9 elems)
    pub boards: [Board; 9],
    pub states: [State; 9],
    /// Indicates active board, 0-8 is the idx, 9 means any board is free
    pub active: usize,
}

/// Template for board, a-i represents which board
/// x means insert fmt char, y means insert reset char
const TEMPLATE: &[u8] = b"
  \x1b[0;3nma | a | a\x1b[0m   |   \x1b[0;3omb | b | b\x1b[0m   |   \x1b[0;3pmc | c | c\x1b[0m   
 \x1b[0;3nm-----------\x1b[0m  |  \x1b[0;3om-----------\x1b[0m  |  \x1b[0;3pm-----------\x1b[0m  
  \x1b[0;3nma | a | a\x1b[0m   |   \x1b[0;3omb | b | b\x1b[0m   |   \x1b[0;3pmc | c | c\x1b[0m   
 \x1b[0;3nm-----------\x1b[0m  |  \x1b[0;3om-----------\x1b[0m  |  \x1b[0;3pm-----------\x1b[0m  
  \x1b[0;3nma | a | a\x1b[0m   |   \x1b[0;3omb | b | b\x1b[0m   |   \x1b[0;3pmc | c | c\x1b[0m   
              |               |               
---------------------------------------------
              |               |
  \x1b[0;3qmd | d | d\x1b[0m   |   \x1b[0;3rme | e | e\x1b[0m   |   \x1b[0;3smf | f | f\x1b[0m   
 \x1b[0;3qm-----------\x1b[0m  |  \x1b[0;3rm-----------\x1b[0m  |  \x1b[0;3sm-----------\x1b[0m  
  \x1b[0;3qmd | d | d\x1b[0m   |   \x1b[0;3rme | e | e\x1b[0m   |   \x1b[0;3smf | f | f\x1b[0m   
 \x1b[0;3qm-----------\x1b[0m  |  \x1b[0;3rm-----------\x1b[0m  |  \x1b[0;3sm-----------\x1b[0m  
  \x1b[0;3qmd | d | d\x1b[0m   |   \x1b[0;3rme | e | e\x1b[0m   |   \x1b[0;3smf | f | f\x1b[0m   
              |               |               
---------------------------------------------
              |               |
  \x1b[0;3tmg | g | g\x1b[0m   |   \x1b[0;3umh | h | h\x1b[0m   |   \x1b[0;3vmi | i | i\x1b[0m   
 \x1b[0;3tm-----------\x1b[0m  |  \x1b[0;3um-----------\x1b[0m  |  \x1b[0;3vm-----------\x1b[0m  
  \x1b[0;3tmg | g | g\x1b[0m   |   \x1b[0;3umh | h | h\x1b[0m   |   \x1b[0;3vmi | i | i\x1b[0m   
 \x1b[0;3tm-----------\x1b[0m  |  \x1b[0;3um-----------\x1b[0m  |  \x1b[0;3vm-----------\x1b[0m  
  \x1b[0;3tmg | g | g\x1b[0m   |   \x1b[0;3umh | h | h\x1b[0m   |   \x1b[0;3vmi | i | i\x1b[0m   
";

impl Game {
    pub fn new() -> Self {
        Game {
            boards: [Board::new(); 9],
            states: [State::Undecided; 9],
            active: 9,
        }
    }

    pub fn test() -> Self {
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

    pub fn make_move(
        &mut self,
        mv: Move,
        side: Slot,
    ) -> Result<(), &'static str> {
        if self.states[mv.game] != State::Undecided {
            return Err("That game has been finished");
        }

        let brd = &mut self.boards[mv.game];
        if brd[mv.index] != Slot::Empty {
            return Err("square is not empty");
        }

        if self.active == 9 {
            self.active = mv.game;
        }

        if self.active != mv.game {
            return Err("must play in the active board");
        }

        brd[mv.index] = side;
        if won_for(*brd, Slot::X) {
            self.states[mv.game] = State::Won;
        } else if won_for(*brd, Slot::O) {
            self.states[mv.game] = State::Lost;
        } else if brd.full() {
            self.states[mv.game] = State::Tied;
        }

        if self.states[mv.index] != State::Undecided {
            self.active = 9;
        } else {
            self.active = mv.index;
        }

        Ok(())
    }

    pub fn print(&self) -> String {
        let mut res = TEMPLATE.to_vec();
        let mut idxs = [0; 9];

        for byte in res.iter_mut() {
            if byte.is_ascii_alphabetic() {
                match byte {
                    b'n'..=b'v' => {
                        let idx = (*byte - b'n') as usize;
                        if self.states[idx] == State::Undecided
                            && (idx == self.active || self.active == 9)
                        {
                            *byte = b'5';
                        } else if self.states[idx] == State::Lost {
                            *byte = b'2';
                        } else if self.states[idx] == State::Won {
                            *byte = b'1';
                        } else if self.states[idx] == State::Tied {
                            *byte = b'3';
                        } else {
                            *byte = b'7';
                        }
                    }
                    b'm' => continue,
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
