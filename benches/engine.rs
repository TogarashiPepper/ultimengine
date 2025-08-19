use divan::Bencher;
use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use ultimengine::{
    board::{Slot, State},
    counting::{alpha_beta, score_game},
    game::Game,
    moves::legal_moves,
};

fn main() {
    divan::main();
}

#[divan::bench]
fn one_game(bencher: Bencher) {
    bencher
        .with_inputs(|| (Game::new(), SmallRng::seed_from_u64(42)))
        .bench_values(|(mut game, mut rng)| {
            loop {
                if game.state != State::Undecided {
                    break;
                }

                let mv = alpha_beta(&game);

                game.make_move(mv.1, Slot::X).unwrap();

                if game.state != State::Undecided {
                    break;
                }

                let legals = legal_moves(&game);

                game.make_move(*legals.choose(&mut rng).unwrap(), Slot::O)
                    .unwrap();
            }
        });
}

#[divan::bench(name = "1000x score_game")]
fn thousand_scores(bencher: Bencher) {
    bencher
        .with_inputs(|| Game::random(40))
        .bench_values(|game| {
            for _ in 0..1000 {
                score_game(&game, Slot::X);
                score_game(&game, Slot::O);
            }
        });
}

#[divan::bench]
fn one_move(bencher: Bencher) {
    bencher
        .with_inputs(|| Game::random(20))
        .bench_values(|mut game| {
            let mv = alpha_beta(&game).1;

            game.make_move(mv, Slot::X).unwrap();
        });
}
