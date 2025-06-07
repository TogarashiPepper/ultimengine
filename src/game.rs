use crate::{
    board::{Board, Slot, State},
    counting::won_for,
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

    pub fn make_move(
        &mut self,
        (board, idx): (usize, usize),
        side: Slot,
    ) -> Result<(), &'static str> {
        if self.active == 9 {
            self.active = board;
        }

        if self.active != board {
            return Err("must play in the active board");
        }

        let brd = &mut self.boards[board];
        if brd[idx] != Slot::Empty {
            return Err("square is not empty");
        }

        if self.states[board] != State::Undecided {
            return Err("That game has been finished");
        }

        brd[idx] = side;
        if won_for(*brd, Slot::X) {
            self.states[board] = State::Won;
        } else if won_for(*brd, Slot::O) {
            self.states[board] = State::Lost;
        } else if brd.full() {
            self.states[board] = State::Tied;
        }

        if self.states[idx] != State::Undecided {
            self.active = 9;
        } else {
            self.active = idx;
        }

        Ok(())
    }

    // Game, Idx
    pub fn parse_move(&self, movestr: &str) -> Result<(usize, usize), &'static str> {
        if movestr.len() > 2 || movestr.is_empty() {
            return Err("Move string must be 1 or 2 chars");
        }

        // TODO: add proper error handling and make dr
        if movestr.len() == 1 && self.active != 9 {
            return Ok((
                self.active,
                movestr.as_bytes()[0] as usize - '0' as usize - 1,
            ));
        }

        // Purposefully Invalid sentinel
        let mut mov = (9, 9);
        let [game, idx] = movestr.as_bytes() else {
            unreachable!()
        };

        if ('a'..='i').contains(&char::from(*game).to_ascii_lowercase()) {
            mov.0 = *game as usize - 'a' as usize;
        }

        if char::from(*idx).is_ascii_digit() {
            mov.1 = *idx as usize - '0' as usize - 1;
        }

        if mov.0 == 9 || mov.1 == 9 {
            return Err("invalid row or column");
        }

        Ok(mov)
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
