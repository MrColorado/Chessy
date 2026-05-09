use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
};
use super::board::Board;
use super::movegen::gen_moves;
use super::mv::Move;
use super::piece::Color;
use super::rule;

pub fn search(mut board: Board, stop_flag: Arc<AtomicBool>, tx: Sender<Option<Move>>) {
    let color_factor = if board.get_current_player_color() == Color::White { 1 } else { -1 };
    let mut best_move = None;
    let mut depth = 1;

    loop {
        let mut best_score_at_depth = i32::MIN;
        let mut best_move_at_depth = None;

        for mv in gen_moves(&board) {
            let moving_color = board.get_current_player_color();
            let state = board.make_move(mv);
            if rule::is_in_check(&board, moving_color) {
                board.unmake_move(mv, state);
                continue;
            }

            let score = -negamax(&mut board, &stop_flag, depth - 1, -color_factor);
            board.unmake_move(mv, state);

            if stop_flag.load(Ordering::Relaxed) {
                tx.send(best_move).ok();
                return;
            }

            if score > best_score_at_depth {
                best_score_at_depth = score;
                best_move_at_depth = Some(mv);
            }
        }

        best_move = best_move_at_depth;
        if let Some(mv) = best_move_at_depth {
            log::debug!("[search] depth {depth} best={mv} score={best_score_at_depth}");
        }
        depth += 1;
    }
}

fn negamax(board: &mut Board, stop_flag: &AtomicBool, depth: i32, color_factor: i32) -> i32 {
    if stop_flag.load(Ordering::Relaxed) {
        return 0;
    }

    if board.get_halfmove_clock() >= 100 {
        return 0;
    }

    if depth == 0 {
        return color_factor * board.get_board_evaluation();
    }

    let color = board.get_current_player_color();
    let mut best_score = i32::MIN + 1;
    let mut has_legal_move = false;
    for mv in gen_moves(board) {
        let state = board.make_move(mv);
        if rule::is_in_check(board, color) {
            board.unmake_move(mv, state);
            continue;
        }
        has_legal_move = true;
        best_score = best_score.max(-negamax(board, stop_flag, depth - 1, -color_factor));
        board.unmake_move(mv, state);
    }

    if !has_legal_move {
        return if rule::is_in_check(board, color) { i32::MIN + 1 } else { 0 };
    }

    best_score
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use crate::engine::board::Board;

    fn eval(fen: &str, depth: i32, color_factor: i32) -> i32 {
        let stop = AtomicBool::new(false);
        let mut board = Board::from_fen(fen).unwrap();
        negamax(&mut board, &stop, depth, color_factor)
    }

    #[test]
    fn balanced_position_scores_zero_at_depth_0() {
        // Position avec matériel égal, éval doit être 0
        assert_eq!(eval("k7/8/8/8/8/8/8/K7 w - - 0 1", 0, 1), 0);
    }

    #[test]
    fn captures_free_rook() {
        // Position équilibrée (tour vs tour). Blanc peut capturer la tour noire en g1.
        // Après Rxg1 : blanc K+R, noir K → score = +500
        assert!(eval("k7/8/8/8/8/8/8/KR4r1 w - - 0 1", 1, 1) >= 500);
    }

    #[test]
    fn mate_in_one_is_max_score() {
        // Blanc joue Tb8# : pion a7 couvre b8, Ka6 couvre a7 et b7 → mat
        assert_eq!(eval("k7/P7/KR6/8/8/8/8/8 w - - 0 1", 2, 1), i32::MAX);
    }

    #[test]
    fn stalemate_scores_zero() {
        // Noir à jouer, roi en a8, aucun coup légal mais pas en échec → nulle
        // Ka6 et Qb6 piègent le roi sans l'attaquer directement
        assert_eq!(eval("k7/8/KQ6/8/8/8/8/8 b - - 0 1", 1, -1), 0);
    }

    #[test]
    fn fifty_move_rule_scores_zero() {
        // Horloge à 100 demi-coups → nulle immédiate quelle que soit la position
        assert_eq!(eval("k7/P7/KR6/8/8/8/8/8 w - - 100 1", 2, 1), 0);
    }
}
