mod board;
mod counting;
mod game;
mod moves;
mod ref_counting;

use cfg_if::cfg_if;
use moves::Move;
use rand::Rng;
use std::{
    cmp::{Ordering, max, min},
    collections::BTreeMap,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

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
    let mut considered_scores: BTreeMap<i32, Vec<Move>> = BTreeMap::new();

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

        let mut mv = Move {
            game: 99,
            index: 99,
        };

        last_eng_score = alpha_beta(&game, &mut mv, 0, i32::MIN, i32::MAX, true);
        last_g = game.clone();

        game.make_move(mv, Slot::X).unwrap();

        redraw(&game);
    }
}

const MAX_DEPTH: u8 = 9;

fn alpha_beta(
    game: &Game,
    choice: &mut Move,
    depth: u8,
    mut alp: i32,
    mut bet: i32,
    is_max: bool,
) -> i32 {
    if depth == MAX_DEPTH || game.state != State::Undecided {
        return score_game(game, if is_max { Slot::O } else { Slot::X });
    }

    if is_max {
        let mut value = i32::MIN;
        let lgs = legal_moves(game);

        for legal in lgs {
            let sim = game.sim_move(legal, Slot::X).unwrap();
            let eval = alpha_beta(
                &sim,
                choice,
                min(depth + 1 + (sim.active == 9) as u8, MAX_DEPTH),
                alp,
                bet,
                false,
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
        let lgs = legal_moves(game);

        for legal in lgs {
            let sim = game.sim_move(legal, Slot::O).unwrap();
            let eval = alpha_beta(
                &sim,
                choice,
                min(depth + 1 + 2 * (sim.active == 9) as u8, MAX_DEPTH),
                alp,
                bet,
                true,
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

#[cfg(feature = "benchmark")]
fn main() {
    let mut handles = vec![];
    let (tx, rx): (Sender<State>, Receiver<State>) = std::sync::mpsc::channel();

    for _ in 0..10 {
        let tx = tx.clone();

        let handle = std::thread::spawn(move || {
            let mut rng = rand::rng();
            let mut outcomes = [State::Undecided; 10000];

            for outcome in &mut outcomes {
                let mut game = Game::new();

                loop {
                    if game.state != State::Undecided {
                        break;
                    }

                    // let mv = legal_moves(&game)
                    //     .into_iter()
                    //     .map(|mv| {
                    //         (
                    //             mv,
                    //             score_game(&game.sim_move(mv, Slot::X).unwrap(), Slot::X),
                    //         )
                    //     })
                    //     .reduce(|acc, cur| match acc.1.cmp(&cur.1) {
                    //         Ordering::Less => cur,
                    //         Ordering::Greater => acc,
                    //         Ordering::Equal => {
                    //             let rn: bool = rng.random();
                    //             if rn { acc } else { cur }
                    //         }
                    //     })
                    //     .unwrap()
                    //     .0;
                    let mut mv = Move {
                        game: 99,
                        index: 99,
                    };

                    alpha_beta(&game, &mut mv, 0, i32::MIN, i32::MAX, true);

                    game.make_move(mv, Slot::X).unwrap();

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
                tx.send(game.state).unwrap();
            }

            outcomes
        });

        handles.push(handle);
    }

    let mut won = 0.0;
    let mut loss = 0.0;
    let mut tied = 0.0;
    for x in 0..100_000 {
        let oc = rx.recv().unwrap();

        match oc {
            State::Won => won += 1.0,
            State::Lost => loss += 1.0,
            State::Tied => tied += 1.0,
            State::Undecided => {}
        }

        println!(
            "{:0.3}% ({x}): finished (state: {:?})",
            x as f64 / 100_000.0 * 100.0,
            oc
        );
        println!(
            "win%: {:0.3} ({won}), loss%: {:0.3} ({loss}), tie%: {:0.3} ({tied})\n",
            won / x as f64 * 100.0,
            loss / x as f64 * 100.0,
            tied / x as f64 * 100.0
        )
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
