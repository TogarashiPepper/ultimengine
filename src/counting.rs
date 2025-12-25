use std::cmp::{max, min};

use crate::{
	bitboard::{
		BitBoard,
		consts::{O_MASK, X_MASK},
	},
	board::{Slot, State},
	game::Game,
	generated::POSSIBLE_TO_WIN,
	moves::{Move, legal_moves},
};

pub fn alpha_beta(game: &Game) -> (i32, Move) {
	// 15 is the highest that fits in the u4 of storage for each field
	let mut mv = Move::new(15, 15);

	let num_moves_made: u32 = game
		.boards
		.map(|b| (b.0 & X_MASK).count_ones() + (b.0 & O_MASK).count_ones())
		.iter()
		.sum();

	let scr = if num_moves_made >= 14 {
		_alpha_beta::<true, 13>(game, &mut mv, 0, i32::MIN, i32::MAX)
	} else {
		_alpha_beta::<true, 11>(game, &mut mv, 0, i32::MIN, i32::MAX)
	};

	(scr, mv)
}

fn _alpha_beta<const IS_MAX: bool, const MAX_DEPTH: u8>(
	game: &Game,
	choice: &mut Move,
	depth: u8,
	mut alp: i32,
	mut bet: i32,
) -> i32 {
	if depth >= MAX_DEPTH || game.state() != State::Undecided {
		return score_game(game, if IS_MAX { Slot::O } else { Slot::X });
	}

	let mut lgs = legal_moves(game);

	if IS_MAX {
		let mut value = i32::MIN;

		// Scoring the games to sort them is costly
		// but alpha-beta pruning benefits so much from
		// a sorted list that it's worth it (see sortdepthanalysis.txt)
		if depth <= 8 {
			lgs.sort_unstable_by(|a, b| {
				let asim = game.sim_move(*a, Slot::X).unwrap();
				let bsim = game.sim_move(*b, Slot::X).unwrap();

				score_game(&bsim, Slot::X).cmp(&score_game(&asim, Slot::X))
			});
		}

		for legal in lgs {
			let sim = game.sim_move(legal, Slot::X).unwrap();
			// TODO: use table

			let eval = _alpha_beta::<false, MAX_DEPTH>(
				&sim,
				choice,
				depth + 1 + (sim.active == 9) as u8,
				alp,
				bet,
			);

			if eval > value && depth == 0 {
				*choice = legal;
			}
			value = max(value, eval);

			if value >= bet {
				break;
			}
			alp = max(alp, value);
		}

		value
	} else {
		let mut value = i32::MAX;

		if depth <= 8 {
			lgs.sort_unstable_by(|a, b| {
				let asim = game.sim_move(*a, Slot::O).unwrap();
				let bsim = game.sim_move(*b, Slot::O).unwrap();

				score_game(&bsim, Slot::O).cmp(&score_game(&asim, Slot::O))
			});
		}

		for legal in lgs {
			let sim = game.sim_move(legal, Slot::O).unwrap();
			let eval = _alpha_beta::<true, MAX_DEPTH>(
				&sim,
				choice,
				depth + 1 + 2 * (sim.active == 9) as u8,
				alp,
				bet,
			);

			if eval < value && depth == 0 {
				*choice = legal;
			}
			value = min(value, eval);

			if value <= alp {
				break;
			}
			bet = min(bet, value);
		}

		value
	}
}

pub fn score_game(game: &Game, turn: Slot) -> i32 {
	let mut scr = score(game.shrink(), turn) * 100;

	scr += game.boards.map(|b| score(b, turn) / 4).iter().sum::<i32>();

	for st in game.boards.map(BitBoard::state) {
		if st == State::Won {
			scr += 100;
		} else if st == State::Lost {
			scr -= 100;
		}
	}

	if game.active == 9 && turn == Slot::X {
		scr -= scr / 3;
	} else if game.active == 9 && turn == Slot::O {
		scr += scr / 3;
	}

	scr
}

// Takes a `Board` and returns a "score" for how good it is for `X`
#[inline]
pub fn score(board: BitBoard, turn: Slot) -> i32 {
	if board.won_by_x() {
		return 10_000;
	} else if board.won_by_o() {
		return -10_000;
	}

	let mut score = 0;

	// We like corners because they open up the ability to make diagonals,
	// something which top mid, bot mid and the two sides dont let us do
	score += board.corners(Slot::X);
	score -= board.corners(Slot::O);

	if turn == Slot::X {
		score += 6 * board.one_aways_x();
		score += 3 * board.one_aways_o();
	} else if turn == Slot::O {
		score -= 6 * board.one_aways_o();
		score -= 3 * board.one_aways_x();
	}

	score
}

#[cfg(not(all(target_arch = "aarch64", target_feature = "neon")))]
pub const fn possible_to_win(board: BitBoard) -> bool {
	let mut idx = 0;

	loop {
		if idx == 88 {
			break false;
		}

		let b = POSSIBLE_TO_WIN[idx];

		if b == (b & board.0) {
			break true;
		}

		idx += 1;
	}
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
#[inline]
pub fn possible_to_win(board: BitBoard) -> bool {
	use std::arch::aarch64::{
		vandq_u32, vceqq_u32, vld1q_dup_u32, vld1q_u32_x4, vmaxvq_u32, vorrq_u32,
	};

	let mut idx = 0;
	let brd = unsafe { vld1q_dup_u32(&raw const board.0) };

	loop {
		if idx >= 88 {
			break false;
		}

		let max = unsafe {
			let masks = vld1q_u32_x4(POSSIBLE_TO_WIN.as_ptr().add(idx));

			let and0 = vandq_u32(masks.0, brd);
			let and1 = vandq_u32(masks.1, brd);
			let and2 = vandq_u32(masks.2, brd);
			let and3 = vandq_u32(masks.3, brd);

			let eqs0 = vceqq_u32(and0, masks.0);
			let eqs1 = vceqq_u32(and1, masks.1);
			let eqs2 = vceqq_u32(and2, masks.2);
			let eqs3 = vceqq_u32(and3, masks.3);

			let comb01 = vorrq_u32(eqs0, eqs1);
			let comb23 = vorrq_u32(eqs2, eqs3);
			let comb = vorrq_u32(comb01, comb23);

			vmaxvq_u32(comb)
		};

		if max > 0 {
			return true;
		}

		idx += 16
	}
}
