use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use chess::engine::board::Board;
use chess::engine::movegen::gen_legal_moves;
use chess::engine::rule::perft;
use chess::engine::searcher::negamax_bench;

fn startpos() -> Board {
    Board::new()
}

fn kiwipete() -> Board {
    Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
        .unwrap()
}

fn pos3() -> Board {
    Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap()
}

// ── génération de coups légaux ───────────────────────────────────────────────

fn bench_gen_legal_moves(c: &mut Criterion) {
    let positions = [
        ("startpos", startpos()),
        ("kiwipete", kiwipete()),
        ("pos3", pos3()),
    ];
    let mut g = c.benchmark_group("gen_legal_moves");
    for (name, board) in positions {
        g.bench_with_input(BenchmarkId::from_parameter(name), &board, |b, board| {
            let mut board = *board;
            b.iter(|| gen_legal_moves(black_box(&mut board)))
        });
    }
    g.finish();
}

// ── perft (exploration pure de l'arbre) ──────────────────────────────────────

fn bench_perft(c: &mut Criterion) {
    let mut g = c.benchmark_group("perft");
    g.sample_size(10);

    g.bench_function("startpos/d4", |b| {
        b.iter(|| perft(black_box(&mut startpos()), 4))
    });
    g.bench_function("kiwipete/d3", |b| {
        b.iter(|| perft(black_box(&mut kiwipete()), 3))
    });
    g.bench_function("pos3/d4", |b| {
        b.iter(|| perft(black_box(&mut pos3()), 4))
    });

    g.finish();
}

// ── recherche negamax ─────────────────────────────────────────────────────────

fn bench_search(c: &mut Criterion) {
    let mut g = c.benchmark_group("negamax");
    g.sample_size(10);

    g.bench_function("startpos/d3", |b| {
        b.iter(|| negamax_bench(black_box(&mut startpos()), 3))
    });
    g.bench_function("kiwipete/d3", |b| {
        b.iter(|| negamax_bench(black_box(&mut kiwipete()), 3))
    });

    g.finish();
}

criterion_group!(benches, bench_gen_legal_moves, bench_perft, bench_search);
criterion_main!(benches);
