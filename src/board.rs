use std::fmt::Display;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Slot {
    Empty,
    X,
    O,
}

impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Slot::Empty => ' ',
                Slot::X => 'X',
                Slot::O => 'O',
            }
        )
    }
}

pub type Board = [Slot; 9];

// Takes a `Board` and returns a "score" for how good it is for `X`
// `side` must not be `Slot::Empty`
pub fn score(board: Board, side: Slot) -> Option<i32> {
    use Slot::{O, X, Empty};

    if side == Slot::Empty { return None }
    let mut score = 0;

    // check rows
    for row in board.windows(3) {
        if matches!(row, [X, X, Empty] | [Empty, X, X] | [X, Empty, X]) {
            score = i32::MAX;
        }

        if matches!(row, [O, O, Empty] | [Empty, O, O] | [O, Empty, O]) {
            score = i32::MIN;
        }
    }

    Some(score)
}

pub struct Game {
    pub boards: [Board; 9],
    /// Indicates active board, 0-8 is the idx, 9 means any board is free
    pub active: usize,
}

const TEMPLATE: &[u8] = br#"
  x | x | x   |   x | x | x   |   x | x | x   
 -----------  |  -----------  |  -----------  
  x | x | x   |   x | x | x   |   x | x | x   
 -----------  |  -----------  |  -----------  
  x | x | x   |   x | x | x   |   x | x | x   
              |               |               
---------------------------------------------
              |               |
  x | x | x   |   x | x | x   |   x | x | x   
 -----------  |  -----------  |  -----------  
  x | x | x   |   x | x | x   |   x | x | x   
 -----------  |  -----------  |  -----------  
  x | x | x   |   x | x | x   |   x | x | x   
              |               |               
---------------------------------------------
              |               |
  x | x | x   |   x | x | x   |   x | x | x   
 -----------  |  -----------  |  -----------  
  x | x | x   |   x | x | x   |   x | x | x   
 -----------  |  -----------  |  -----------  
  x | x | x   |   x | x | x   |   x | x | x   
"#;

impl Game {
    pub fn new() -> Self {
        Game {
            boards: [[Slot::Empty; 9]; 9],
            active: 9,
        }
    }

    pub fn print(&self) -> String {
        // SAFETY: arrays are guaranteed to be contiguous in memory so this should be fine
        // (also it passes miri)
        let flat: [Slot; 81] = unsafe { std::mem::transmute(self.boards) };
        let mut res = TEMPLATE.to_vec();
        let mut idx = 0;

        for byte in res.iter_mut() {
            if *byte == b'x' {
                *byte = match flat[idx] {
                    Slot::O => b'O',
                    Slot::X => b'X',
                    Slot::Empty => b' ',
                };

                idx += 1;
            }
        }
        
        String::from_utf8(res).unwrap()
    }
}

