use crate::{
	bitboard::BitBoard,
	board::Slot,
	counting::alpha_beta,
	game::Game,
	moves::{Move, legal_moves},
};

// Stores a vec of game -> refutation move
#[derive(Debug)]
pub struct OpeningBook(pub [(Game, Move); 794]);

pub static OBOOK: OpeningBook = include!("../openingbook.serialized");

impl OpeningBook {
	// TODO: clean this up and make it so you can put in an `n` and get out the opening book for
	// the first `n` moves of the game
	pub fn generate() -> OpeningBook {
		let mut inner = vec![];
		let game = Game::new();

		// Generate refutations for the second move
		for mv in legal_moves(&game) {
			let sub1_game = game.sim_move(mv, Slot::O).unwrap();
			let (_scr, refutation) = alpha_beta(&sub1_game);

			inner.push((sub1_game.clone(), refutation));

			let sub2_game = sub1_game.sim_move(refutation, Slot::X).unwrap();
			let thrd_inner = std::thread::scope(|s| {
				let mut handles = vec![];

				// Generate refutations for the fourth move
				for lg_mv in legal_moves(&sub2_game.clone()) {
					let sub2_game = sub2_game.clone();
					handles.push(s.spawn(move || {
						let sub3_game = sub2_game.sim_move(lg_mv, Slot::O).unwrap();
						let (_scr, refutation) = alpha_beta(&sub3_game);

						(sub3_game, refutation)
					}));
				}

				handles
					.into_iter()
					.map(|h| h.join().unwrap())
					.collect::<Vec<(Game, Move)>>()
			});

			inner.extend_from_slice(&thrd_inner);
		}

		OpeningBook(inner.try_into().unwrap())
	}
}
