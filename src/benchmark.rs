use std::{
    cmp::Ordering,
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

use rand::{Rng, rngs::ThreadRng};

use crate::{
    board::{Slot, State},
    counting::alpha_beta,
    game::Game,
    moves::legal_moves,
    ref_counting::ref_score_game,
};

fn play_game(rng: &mut ThreadRng) -> State {
    let mut game = Game::new();

    loop {
        if game.state != State::Undecided {
            break;
        }

        let mv = alpha_beta(&game).1;

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

    game.state
}

pub fn benchmark() {
    let mut handles = vec![];
    let (tx, rx): (Sender<State>, Receiver<State>) = std::sync::mpsc::channel();
    let start = Instant::now();

    const IS_PERF_FOCUSED: bool = option_env!("BENCH_PERF").is_some();
    const NUM_THREADS: usize = if IS_PERF_FOCUSED { 1 } else { 8 };

    for _ in 0..NUM_THREADS {
        let tx = tx.clone();

        let handle = std::thread::spawn(move || {
            let mut rng = rand::rng();
            let mut outcomes = [State::Undecided; 360 / NUM_THREADS];

            for outcome in &mut outcomes {
                *outcome = play_game(&mut rng);
                tx.send(*outcome).unwrap();
            }

            outcomes
        });

        handles.push(handle);
    }

    let mut won = 0.0;
    let mut loss = 0.0;
    let mut tied = 0.0;

    println!("Waiting for first game to finish, this should only take a few seconds.");

    loop {
        let x = (won + loss + tied) as u128;
        if x >= 360 {
            break;
        }

        std::thread::sleep(Duration::from_millis(400));
        let elapsed = start.elapsed();

        if let Ok(oc) = rx.try_recv() {
            match oc {
                State::Won => won += 1.0,
                State::Lost => loss += 1.0,
                State::Tied => tied += 1.0,
                State::Undecided => unreachable!(),
            }
        }

        if x == 0 {
            continue;
        }

        print!("\x1B[2J\x1B[1;1H");
        println!(
            "time spent: {}s, estimated time remaining: {}s (avg. time/game: {:0.2}s)",
            elapsed.as_secs(),
            (360 - x) * (elapsed.as_millis() / x) / 1000,
            elapsed.as_secs() as f64 / x as f64,
        );
        println!("{:0.3}% ({x}): finished", x as f64 / 360.0 * 100.0,);
        println!(
            "win%: {:0.3} ({won}), loss%: {:0.3} ({loss}), tie%: {:0.3} ({tied})",
            won / x as f32 * 100.0,
            loss / x as f32 * 100.0,
            tied / x as f32 * 100.0
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
