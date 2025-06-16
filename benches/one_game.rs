use std::time::Duration;

use criterion::{BatchSize, Criterion, SamplingMode, criterion_group, criterion_main};
use rand::seq::IndexedRandom;
use ultimengine::{
    board::{Slot, State},
    counting::{alpha_beta, score_game},
    game::Game,
    moves::{Move, legal_moves},
};

pub fn bench_alphabeta(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_beta pruning bench");
    group
        .sampling_mode(SamplingMode::Flat)
        .sample_size(50)
        .measurement_time(Duration::from_secs(150));

    group.bench_function("one game", |b| {
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

                alpha_beta::<true>(&game, &mut mv, 0, i32::MIN, i32::MAX);

                game.make_move(mv, Slot::X).unwrap();

                if game.state != State::Undecided {
                    break;
                }

                let legals = legal_moves(&game);

                game.make_move(*legals.choose(&mut rng).unwrap(), Slot::O)
                    .unwrap();
            }
        });
    });

    group.finish();
}

pub fn bench_scoring(c: &mut Criterion) {
    c.bench_function("10x score_game", |b| {
        b.iter_batched(
            || Game::random(40),
            |game| {
                for _ in 0..1000 {
                    score_game(&game, Slot::X);
                }
            },
            BatchSize::SmallInput,
        )
    });
}

pub fn bench_one_move(c: &mut Criterion) {
    c.bench_function("one move", |b| b.iter_batched(|| Game::random(20), |mut g| {
        let mut mv = Move {
            game: 99,
            index: 99,
        };

        alpha_beta::<true>(&g, &mut mv, 0, i32::MIN, i32::MAX);

        g.make_move(mv, Slot::X).unwrap();
    }, BatchSize::SmallInput));
}

criterion_group!(benches, bench_scoring, bench_one_move, bench_alphabeta);
criterion_main!(benches);
