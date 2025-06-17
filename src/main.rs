mod board;
mod counting;
mod game;
mod moves;
mod ref_counting;
mod benchmark;

use cfg_if::cfg_if;

use std::
    time::Instant
;

use crate::{
    board::{Slot, State},
    counting::alpha_beta,
    game::Game,
    moves::{Move, parse_move},
};

fn redraw(game: &Game) {
    print!("\x1B[2J\x1B[1;1H");
    println!("{}", game.print());

    match game.state {
        State::Won => println!("YOU HAVE LOST!!!!!"),
        State::Lost => println!("YOU HAVE WON!!!!!"),
        State::Tied => println!("tie game :("),
        State::Undecided => return,
    }

    std::process::exit(1);
}

#[cfg(not(feature = "benchmark"))]
fn main() {
    #[cfg(feature = "savestates")]
    let config = bincode::config::standard();
    let stdin = std::io::stdin();

    let instant = Instant::now();

    let mut game = if std::env::var("LOAD_GAME").is_ok() {
        cfg_if! {
            if #[cfg(feature = "savestates")] {
                let mut file = std::fs::File::open("gamestate").unwrap();

                bincode::decode_from_std_read(&mut file, config).unwrap()
            }
            else {
                panic!("This binary has not been compiled to use savestates")
            }
        }
    } else {
        Game::new()
    };

    let mut mov_buf = String::new();
    let mut last_eng_score = 0;
    let mut last_g = Game::new();

    loop {
        redraw(&game);

        print!(
            "Enter your move (ex. a5, active board: {}): ",
            if game.active == 9 {
                ' '
            } else {
                (game.active as u8 + b'a') as char
            }
        );

        use std::io::Write;
        std::io::stdout().flush().unwrap();

        mov_buf.clear();
        stdin.read_line(&mut mov_buf).unwrap();

        match mov_buf.trim() {
            "undo" => std::mem::swap(&mut game, &mut last_g),
            #[cfg(feature = "savestates")]
            x @ ("save" | "undosave") => {
                let mut file = std::fs::File::create("gamestate").unwrap();

                if x == "save" {
                    bincode::encode_into_std_write(game.clone(), &mut file, config).unwrap();
                } else {
                    bincode::encode_into_std_write(last_g.clone(), &mut file, config).unwrap();
                }

                std::process::exit(0);
            }
            "skip" => {}
            _ => {
                let mv = parse_move(mov_buf.trim(), game.active)
                    .and_then(|mv| game.make_move(mv, Slot::O));
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

        redraw(&game);

        let (scr, mv) = alpha_beta(&game);

        last_eng_score = scr;
        last_g = game.clone();

        game.make_move(mv, Slot::X).unwrap();

        redraw(&game);
    }
}

#[cfg(feature = "benchmark")]
fn main() {
    benchmark::benchmark();
}
