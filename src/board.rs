use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Slot {
    Empty,
    X,
    O,
}

impl Slot {
    fn to_chr(self) -> char {
        match self {
            Slot::Empty => ' ',
            Slot::X => 'X',
            Slot::O => 'O',
        }
    }
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_chr())
    }
}

pub type Board = [Slot; 9];

fn tranpose(board: &Board) -> Board {
    let mut new = *board;

    new.swap(3, 1);
    new.swap(6, 2);
    new.swap(5, 7);

    new
}

// TODO: consolidate functions into one
fn one_away_x(line: &[Slot; 3]) -> bool {
    use Slot::{Empty, X};

    matches!(line, [X, X, Empty] | [Empty, X, X] | [X, Empty, X])
}

fn one_away_o(line: &[Slot; 3]) -> bool {
    use Slot::{Empty, O};

    matches!(line, [O, O, Empty] | [Empty, O, O] | [O, Empty, O])
}

// Takes a `Board` and returns a "score" for how good it is for `X`
// `side` must not be `Slot::Empty`
pub fn score(board: Board) -> Option<i32> {
    let mut score = 0;

    for row in board.windows(3) {
        let row = row.try_into().unwrap();
        if one_away_x(row) {
            score = i32::MAX;
        }

        if one_away_o(row) {
            score = i32::MIN;
        }
    }

    for column in tranpose(&board).windows(3) {
        let column = column.try_into().unwrap();

        if one_away_x(column) {
            score = i32::MAX;
        }

        if one_away_o(column) {
            score = i32::MIN;
        }
    }

    for diag in [
        [board[0], board[4], board[8]],
        [board[2], board[4], board[6]],
    ] {
        if one_away_x(&diag) {
            score = i32::MAX;
        }

        if one_away_o(&diag) {
            score = i32::MIN;
        }
    }

    Some(score)
}

pub fn full(board: &Board) -> bool {
    board.iter().all(|sl| *sl != Slot::Empty)
}

#[derive(Debug, Clone, Copy)]
enum State {
    Won,
    Lost,
    Tied,
    Undecided,
}

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
            boards: [[Slot::Empty; 9]; 9],
            states: [State::Undecided; 9],
            active: 9,
        }
    }

    pub fn print(&self) -> String {
        // SAFETY: arrays are guaranteed to be contiguous in memory so this should be fine
        // (also it passes miri)
        // let flat: [Slot; 81] = unsafe { std::mem::transmute(self.boards) };
        let mut res = TEMPLATE.to_vec();
        let mut idxs = [0; 9];

        for byte in res.iter_mut() {
            if byte.is_ascii_alphabetic() {
                match byte {
                    b'x' => *byte = b'4',
                    b'n'..=b'v' => {
                        // *byte = *byte - b'n' + b'1';
                        if (*byte - b'n') as usize == self.active || self.active == 9 {
                            *byte = b'5';
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
