mod board;
mod counting;
mod game;
mod moves;

use std::time::Duration;

use board::{Slot, State};
use counting::{score_game, won_for};
use game::Game;
use moves::{legal_moves, parse_move};
use rand::Rng;

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

const DBG_INFO: bool = true;

fn main() {
    let config = bincode::config::standard();

    let mut game = if std::env::var("LOAD_GAME").is_ok() {
        let mut file = std::fs::File::open("gamestate").unwrap();

        bincode::decode_from_std_read(&mut file, config).unwrap()
    } else {
        Game::new()
    };
    let mut rng = rand::rng();

    let mut mov_buf = String::new();

    let mut last_eng_score = 0;
    let stdin = std::io::stdin();
    let mut last_g = Game::new();

    loop {
        redraw(&game);

        if DBG_INFO {
            use std::fmt::Write;

            let mut bf = String::new();

            write!(bf, "won: ").unwrap();
            for b in game.boards {
                write!(bf, "{:?}, ", won_for(b, Slot::X)).unwrap();
            }
            write!(bf, "\nengine score for it's last move: {last_eng_score}").unwrap();

            println!("{bf}");
        }

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

        let legals = legal_moves(&game)
            .iter()
            .copied()
            .map(|mv| (mv, score_game(&game.sim_move(mv, Slot::X).unwrap())))
            .reduce(|acc, cur| {
                if acc.1 > cur.1 {
                    acc
                } else if acc.1 == cur.1 {
                    let rn: bool = rng.random();
                    if rn { acc } else { cur }
                } else {
                    cur
                }
            })
            .unwrap();

        last_eng_score = legals.1;

        last_g = game.clone();

        game.make_move(legals.0, Slot::X).unwrap();

        std::thread::sleep(Duration::from_secs(1));

        redraw(&game);
    }
}
