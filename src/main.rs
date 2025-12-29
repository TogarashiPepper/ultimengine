use mimalloc::MiMalloc;
use ultimengine::{
	board::{Slot, State},
	counting::engine_mv,
	game::Game,
	moves::{Move, parse_move},
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn redraw(game: &Game, last_mv: Option<Move>) {
	print!("\x1B[2J\x1B[1;1H");
	println!("{}", game.print(last_mv));

	match game.state() {
		State::Won => println!("YOU HAVE LOST!!!!!"),
		State::Lost => println!("YOU HAVE WON!!!!!"),
		State::Tied => println!("tie game :("),
		State::Undecided => return,
	}

	std::process::exit(1);
}

fn main() {
	let stdin = std::io::stdin();

	let mut game = Game::new();

	let mut mov_buf = String::new();
	let mut last_g = Game::new();
	let mut last_mv = None;

	loop {
		redraw(&game, last_mv);

		print!(
			"Enter your move (ex. a5, active board: {}): ",
			if game.active == 9 {
				' '
			} else {
				(game.active + b'a') as char
			}
		);

		use std::io::Write;
		std::io::stdout().flush().unwrap();

		mov_buf.clear();
		stdin.read_line(&mut mov_buf).unwrap();

		match mov_buf.trim() {
			"undo" => std::mem::swap(&mut game, &mut last_g),
			"skip" => {}
			_ => {
				let mv = parse_move(mov_buf.trim(), game.active).and_then(|mv| {
					let r = game.make_move(mv, Slot::O);
					if r.is_ok() {
						last_mv = Some(mv);
					}

					r
				});

				if let Err(e) = mv {
					println!("\x1b[0;31m{e} (press enter to continue)\x1b[0m");

					// we dont care about what's here so we write to
					// the move buffer because we know it will be overwritten
					// immediately after `continue` is called
					stdin.read_line(&mut mov_buf).unwrap();

					continue;
				}
			}
		}

		redraw(&game, last_mv);

		let mv = engine_mv(&game);

		last_g = game.clone();

		game.make_move(mv, Slot::X).unwrap();

		last_mv = Some(mv);

		redraw(&game, last_mv);
	}
}
