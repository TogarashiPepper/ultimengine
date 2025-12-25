use std::fmt::Debug;

#[cfg(feature = "savestates")]
use bincode::{Decode, Encode};

use crate::{
	bitboard::consts::{E_OFFS, ST_MASK, ST_OFFS},
	board::State,
	game::Game,
};

#[derive(PartialEq, Clone, Copy, Hash, Eq)]
#[cfg_attr(feature = "savestates", derive(Encode, Decode))]
/// Upper 4 bits is .game, lower 4 bits is .index
/// 0000  0000
/// game  idx
pub struct Move(u8);

impl Move {
	pub fn new(game: u8, index: u8) -> Self {
		let idx_b = index & 0b00001111;
		let gam_b = (game & 0b00001111) << 4;
		Move(idx_b | gam_b)
	}

	#[inline]
	pub const fn game(&self) -> u8 {
		self.0 >> 4
	}

	#[inline]
	pub const fn set_game(&mut self, idx: u8) {
		debug_assert!(idx <= 9);

		self.0 &= 0b00001111;
		self.0 |= (idx & 0b00001111) << 4;
	}

	#[inline]
	pub const fn index(&self) -> u8 {
		self.0 & 0b00001111
	}

	#[inline]
	pub const fn set_idx(&mut self, idx: u8) {
		debug_assert!(idx <= 9);

		self.0 &= 0b11110000;
		self.0 |= idx & 0b00001111;
	}
}

impl Debug for Move {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}{}", (self.game() + b'A') as char, self.index() + 1)
	}
}

#[inline]
pub fn is_legal(game: &Game, mv: Move) -> Result<(), &'static str> {
	if game.boards[mv.game() as usize].state() != State::Undecided {
		return Err("That game has been finished");
	}

	let idx = 1 << (18 + mv.index());
	if game.boards[mv.game() as usize].0 & idx != idx {
		return Err("square is not empty");
	}

	if game.active != mv.game() && game.active != 9 {
		return Err("must play in the active board");
	}

	Ok(())
}

// Game, Idx
pub fn parse_move(input: &str, active: u8) -> Result<Move, &'static str> {
	if input.len() > 2 || input.is_empty() {
		return Err("Move string must be 1 or 2 chars");
	}

	if active == 9 && input.len() == 1 {
		return Err("Can only use shorthand notation when a specific board is active");
	}

	if input.len() == 1 && active != 9 {
		return Ok(Move::new(active, input.as_bytes()[0] - b'0' - 1));
	}

	// Purposefully Invalid sentinel
	// game, idx
	let mut mov = Move::new(active, 9);

	let bs = input.as_bytes();

	if input.len() == 2 {
		if !('a'..='i').contains(&char::from(bs[0])) {
			return Err("game must be within a to i");
		};

		let v = bs[0] - b'a';
		mov.set_game(v);
	}

	if !char::from(bs[bs.len() - 1]).is_ascii_digit() {
		return Err("index must be between 1 and 9");
	};

	mov.set_idx(bs[bs.len() - 1] - b'0' - 1);

	Ok(mov)
}

#[inline]
pub fn fast_legal(game: &Game, mv: Move) -> bool {
	let g_idx = mv.game();
	let brd = game.boards[g_idx as usize];

	let in_finished = (brd.0 & ST_MASK) >> ST_OFFS != 0;

	let idx = 1 << (18 + mv.index());
	let in_occupied = brd.0 & idx != idx;

	let not_active = game.active != mv.game() && game.active != 9;

	!(not_active || in_occupied || in_finished)
}

pub fn legal_moves(game: &Game) -> Vec<Move> {
	let mut mvs = Vec::with_capacity(80);

	for bdx in 0..9 {
		for idx in 0..9 {
			let m = Move::new(bdx, idx);

			if fast_legal(game, m) {
				mvs.push(m);
			}
		}
	}

	mvs
}
