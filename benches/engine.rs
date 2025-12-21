use divan::Bencher;
use mimalloc::MiMalloc;
use rand::{SeedableRng, rngs::SmallRng, seq::IndexedRandom};
use ultimengine::{
    board::{Slot, State},
    counting::{alpha_beta, score_game},
    game::Game,
    moves::legal_moves,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {
    divan::main();
}

mod table {
    mod custom {
        use divan::Bencher;
        use rand::random;
        use ultimengine::{board::Slot, counting::score_game, game::Game, table::Table};

        #[divan::bench]
        fn insert_5k(bencher: Bencher) {
            bencher
                .with_inputs(|| {
                    let entries = (0..5_000)
                        .map(|_| {
                            let game = Game::random_seedless(10);
                            let turn = if random() { Slot::X } else { Slot::O };
                            let score = score_game(&game, turn);

                            (game, turn, score)
                        })
                        .collect::<Vec<(Game, Slot, i32)>>();

                    let table = Table::new();

                    (entries, table)
                })
                .bench_values(|(entries, mut table)| {
                    for el in entries {
                        assert!(table.insert((el.0, el.1), el.2));
                    }
                });
        }

        #[divan::bench]
        fn get_5k(bencher: Bencher) {
            bencher
                .with_inputs(|| {
                    let entries = (0..5_000)
                        .map(|_| {
                            let game = Game::random_seedless(10);
                            let turn = if random() { Slot::X } else { Slot::O };
                            let score = score_game(&game, turn);

                            (game, turn, score)
                        })
                        .collect::<Vec<(Game, Slot, i32)>>();

                    let mut table = Table::new();

                    for ent in entries.clone() {
                        table.insert((ent.0, ent.1), ent.2);
                    }

                    (entries, table)
                })
                .bench_values(|(entries, table)| {
                    for el in entries {
                        assert_eq!(table.get(&el.0, el.1), Some(el.2));
                    }
                });
        }
    }

    mod standard {
        use divan::Bencher;
        use rand::random;
        use std::collections::HashMap;
        use ultimengine::{board::Slot, counting::score_game, game::Game};

        #[divan::bench]
        fn insert_5k(bencher: Bencher) {
            bencher
                .with_inputs(|| {
                    let entries = (0..5_000)
                        .map(|_| {
                            let game = Game::random_seedless(10);
                            let turn = if random() { Slot::X } else { Slot::O };
                            let score = score_game(&game, turn);

                            (game, turn, score)
                        })
                        .collect::<Vec<(Game, Slot, i32)>>();

                    let table = HashMap::with_capacity(113636);

                    (entries, table)
                })
                .bench_values(|(entries, mut table)| {
                    for el in entries {
                        assert!(table.insert((el.0, el.1), el.2).is_none());
                    }
                });
        }

        #[divan::bench]
        fn get_5k(bencher: Bencher) {
            bencher
                .with_inputs(|| {
                    let entries = (0..5_000)
                        .map(|_| {
                            let game = Game::random_seedless(10);
                            let turn = if random() { Slot::X } else { Slot::O };
                            let score = score_game(&game, turn);

                            (game, turn, score)
                        })
                        .collect::<Vec<(Game, Slot, i32)>>();

                    let mut table = HashMap::with_capacity(113636);

                    for ent in entries.clone() {
                        table.insert((ent.0, ent.1), ent.2);
                    }

                    (entries, table)
                })
                .bench_values(|(entries, table)| {
                    for el in entries {
                        assert_eq!(table.get(&(el.0, el.1)), Some(&el.2));
                    }
                });
        }
    }
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
