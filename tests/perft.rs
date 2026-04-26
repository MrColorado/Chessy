use chess::engine::board::Board;
use chess::engine::rule::gen_legal_moves;

fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let moves = gen_legal_moves(board);
    if depth == 1 {
        return moves.len() as u64;
    }
    let mut count = 0;
    for &mv in moves.as_slice() {
        let state = board.make_move(mv);
        count += perft(board, depth - 1);
        board.unmake_move(mv, state);
    }
    count
}

fn run_suite(epd: &str, max_depth: u32) {
    let mut failures = Vec::new();

    for (line_idx, line) in epd.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let segments: Vec<&str> = line.split(';').collect();
        let fen = segments[0].trim();

        let mut board = match Board::from_fen(fen) {
            Ok(b) => b,
            Err(e) => {
                failures.push(format!("line {}: bad FEN {:?}: {}", line_idx + 1, fen, e));
                continue;
            }
        };

        for seg in &segments[1..] {
            let seg = seg.trim();
            let mut tokens = seg.split_whitespace();
            let depth_tag = match tokens.next() { Some(t) => t, None => continue };
            let expected_str = match tokens.next() { Some(t) => t, None => continue };

            if !depth_tag.starts_with('D') { continue; }
            let depth: u32 = match depth_tag[1..].parse() {
                Ok(d) => d,
                Err(_) => continue,
            };
            if depth > max_depth { continue; }

            let expected: u64 = match expected_str.parse() {
                Ok(n) => n,
                Err(_) => continue,
            };

            let got = perft(&mut board, depth);
            if got != expected {
                failures.push(format!(
                    "line {} D{}: {} — expected {} got {}",
                    line_idx + 1, depth, fen, expected, got
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!("{} failure(s):\n{}", failures.len(), failures.join("\n"));
    }
}

#[test]
fn perft_suite_standard() {
    run_suite(include_str!("fixtures/standard.epd"), 4);
}
