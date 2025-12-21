use std::fmt::Debug;

#[cfg(feature = "savestates")]
use bincode::{Decode, Encode};

use crate::{board::State, game::Game};

#[derive(PartialEq, Clone, Copy, Hash, Eq)]
#[cfg_attr(feature = "savestates", derive(Encode, Decode))]
pub struct Move {
    pub game: usize,
    pub index: usize,
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", (self.game as u8 + b'A') as char, self.index + 1)
    }
}

#[inline]
pub fn is_legal(game: &Game, mv: Move) -> Result<(), &'static str> {
    if game.boards[mv.game].state() != State::Undecided {
        return Err("That game has been finished");
    }

    let idx = 1 << (18 + mv.index);
    if game.boards[mv.game].0 & idx != idx {
        return Err("square is not empty");
    }

    if game.active != mv.game && game.active != 9 {
        return Err("must play in the active board");
    }

    Ok(())
}

// Game, Idx
pub fn parse_move(input: &str, active: usize) -> Result<Move, &'static str> {
    if input.len() > 2 || input.is_empty() {
        return Err("Move string must be 1 or 2 chars");
    }

    if active == 9 && input.len() == 1 {
        return Err("Can only use shorthand notation when a specific board is active");
    }

    if input.len() == 1 && active != 9 {
        return Ok(Move {
            game: active,
            index: input.as_bytes()[0] as usize - '0' as usize - 1,
        });
    }

    // Purposefully Invalid sentinel
    // game, idx
    let mut mov = Move {
        game: active,
        index: 9,
    };

    let bs = input.as_bytes();

    if input.len() == 2 {
        if !('a'..='i').contains(&char::from(bs[0])) {
            return Err("game must be within a to i");
        };

        let v = bs[0] as usize - 'a' as usize;
        mov.game = v;
    }

    if !char::from(bs[bs.len() - 1]).is_ascii_digit() {
        return Err("index must be between 1 and 9");
    };

    mov.index = bs[bs.len() - 1] as usize - '0' as usize - 1;

    Ok(mov)
}

pub fn legal_moves(game: &Game) -> Vec<Move> {
    let mut mvs = Vec::with_capacity(81);

    for bdx in 0..9 {
        for idx in 0..9 {
            if is_legal(
                game,
                Move {
                    game: bdx,
                    index: idx,
                },
            )
            .is_ok()
            {
                mvs.push(Move {
                    game: bdx,
                    index: idx,
                })
            }
        }
    }

    mvs
}
