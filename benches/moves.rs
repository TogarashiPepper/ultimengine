use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use ultimengine::{board::Slot, counting::alpha_beta, game::Game};

fn moves(c: &mut Criterion) {
	let mut group = c.benchmark_group("moves");

	for mv_count in [2, 20] {
		group.bench_with_input(
			BenchmarkId::from_parameter(mv_count),
			&mv_count,
			|b, count| {
				b.iter_batched(
					|| Game::random(count - 1),
					|mut game| {
						let mv = alpha_beta(&game).1;

						game.make_move(mv, Slot::X).unwrap()
					},
					BatchSize::SmallInput,
				);
			},
		);
	}

	group.finish();
}

criterion_group!(benches, moves);
criterion_main!(benches);
