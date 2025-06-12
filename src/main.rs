mod board;
mod counting;
mod game;
mod moves;
mod ref_counting;

use cfg_if::cfg_if;
use rand::{Rng, seq::IndexedRandom};
use std::{cmp::Ordering, collections::BTreeMap, time::Duration};

use crate::{
    board::{Slot, State},
    counting::score_game,
    game::Game,
    moves::{legal_moves, parse_move},
    ref_counting::ref_score_game,
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

    let mut rng = rand::rng();

    let mut mov_buf = String::new();

    let mut last_eng_score = 0;
    let mut considered_scores = BTreeMap::new();

    let mut last_g = Game::new();

    loop {
        redraw(&game);

        if std::env::var("DEBUG").is_ok() {
            use std::fmt::Write;

            let mut bf = String::new();

            write!(bf, "\nengine score for it's last move: {last_eng_score}").unwrap();
            write!(
                bf,
                "\nengine considered move scores: {considered_scores:#?}"
            )
            .unwrap();
            // write!(bf, "\nconsidered moves: {considered_moves:?}").unwrap();

            considered_scores.clear();

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

        let legals = legal_moves(&game)
            .into_iter()
            .map(|mv| (mv, score_game(&game.sim_move(mv, Slot::X).unwrap())))
            .reduce(|acc, cur| {
                considered_scores
                    .entry(cur.1)
                    .or_insert_with(Vec::new)
                    .push(cur.0);

                match acc.1.cmp(&cur.1) {
                    Ordering::Less => cur,
                    Ordering::Greater => acc,
                    Ordering::Equal => {
                        let rn: bool = rng.random();
                        if rn { acc } else { cur }
                    }
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

#[cfg(feature = "benchmark")]
fn main() {
    let mut handles = vec![];

    for _ in 0..10 {
        let handle = std::thread::spawn(|| {
            let mut rng = rand::rng();
            let mut outcomes = [State::Undecided; 10000];

            for outcome in &mut outcomes {
                let mut game = Game::new();

                loop {
                    if game.state != State::Undecided {
                        break;
                    }

                    let legals = legal_moves(&game)
                        .into_iter()
                        .map(|mv| (mv, score_game(&game.sim_move(mv, Slot::X).unwrap())))
                        .reduce(|acc, cur| match acc.1.cmp(&cur.1) {
                            Ordering::Less => cur,
                            Ordering::Greater => acc,
                            Ordering::Equal => {
                                let rn: bool = rng.random();
                                if rn { acc } else { cur }
                            }
                        })
                        .unwrap();

                    game.make_move(legals.0, Slot::X).unwrap();

                    if game.state != State::Undecided {
                        break;
                    }

                    let flipped = game.flip();
                    let legals = legal_moves(&flipped)
                        .into_iter()
                        .map(|mv| (mv, ref_score_game(&flipped.sim_move(mv, Slot::X).unwrap())))
                        .reduce(|acc, cur| match acc.1.cmp(&cur.1) {
                            Ordering::Less => cur,
                            Ordering::Greater => acc,
                            Ordering::Equal => {
                                let rn: bool = rng.random();
                                if rn { acc } else { cur }
                            }
                        })
                        .unwrap();

                    game.make_move(legals.0, Slot::O).unwrap();
                }

                *outcome = game.state;
            }

            outcomes
        });

        handles.push(handle);
    }

    let mut won = 0;
    let mut loss = 0;
    let mut tied = 0;

    for handle in handles {
        let sub_outcomes = handle.join().unwrap();

        won += sub_outcomes.iter().filter(|e| **e == State::Won).count();
        loss += sub_outcomes.iter().filter(|e| **e == State::Lost).count();
        tied += sub_outcomes.iter().filter(|e| **e == State::Tied).count();
    }

    println!("won: {}, lost: {}, tied: {}", won, loss, tied);
}
