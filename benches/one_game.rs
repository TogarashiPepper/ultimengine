use std::cmp::Ordering;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rand::{Rng, seq::IndexedRandom};
use ultimengine::{
    board::{Slot, State},
    counting::alpha_beta,
    game::Game,
    moves::{Move, legal_moves},
    ref_counting::ref_score_game,
};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("one game", |b| {
        b.iter(|| {
            let mut game = Game::new();
            let mut rng = rand::rng();

            loop {
                if game.state != State::Undecided {
                    break;
                }

                let mut mv = Move {
                    game: 99,
                    index: 99,
                };

                alpha_beta(&game, &mut mv, 0, i32::MIN, i32::MAX, true);

                game.make_move(mv, Slot::X).unwrap();

                if game.state != State::Undecided {
                    break;
                }

                let legals = legal_moves(&game);
                // .into_iter()
                // .map(|mv| (mv, ref_score_game(&flipped.sim_move(mv, Slot::X).unwrap())))
                // .reduce(|acc, cur| match acc.1.cmp(&cur.1) {
                //     Ordering::Less => cur,
                //     Ordering::Greater => acc,
                //     Ordering::Equal => {
                //         let rn: bool = rng.random();
                //         if rn { acc } else { cur }
                //     }
                // })
                // .unwrap();

                game.make_move(*legals.choose(&mut rng).unwrap(), Slot::O)
                    .unwrap();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
