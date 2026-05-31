use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
};

use super::board::Board;
use super::movegen::{gen_legal_moves, gen_legal_captures};
use super::mv::Move;
use super::rule;

const MAX_PLY: usize = 128;

// Marge de sécurité pour le delta pruning : couvre les promotions et les
// petites tactiques courtes qui peuvent regagner du matériel après la capture.
const DELTA_MARGIN: i32 = 200;

pub fn search(mut board: Board, stop_flag: Arc<AtomicBool>, tx: Sender<Option<Move>>) {
    let mut searcher = Searcher::new(stop_flag.clone());
    let mut best_move = None;
    let mut depth = 1;

    loop {
        let mut best_score_at_depth = i32::MIN;
        let mut best_move_at_depth = None;
        let mut alpha = i32::MIN + 1;
        let beta = i32::MAX;

        for mv in gen_legal_moves(&board) {
            let state = board.make_move(mv);
            let score = -searcher.negamax(&mut board, depth - 1, 1, -beta, -alpha);
            board.unmake_move(mv, state);

            if stop_flag.load(Ordering::Relaxed) {
                tx.send(best_move).ok();
                return;
            }

            if score > best_score_at_depth {
                best_score_at_depth = score;
                best_move_at_depth = Some(mv);
                if best_score_at_depth > alpha {
                    alpha = best_score_at_depth;
                }
            }
        }

        best_move = best_move_at_depth;
        if let Some(mv) = best_move_at_depth {
            log::debug!("[search] depth {depth} best={mv} score={best_score_at_depth}");
        }
        depth += 1;
    }
}

pub fn negamax_bench(board: &mut Board, depth: i32) -> i32 {
    let mut searcher = Searcher::new(Arc::new(AtomicBool::new(false)));
    searcher.negamax(board, depth, 0, i32::MIN + 1, i32::MAX)
}

struct Searcher {
    stop_flag: Arc<AtomicBool>,
    killers: [[Option<Move>; 2]; MAX_PLY],
}

impl Searcher {
    pub fn new(stop_flag: Arc<AtomicBool>) -> Searcher {
        Searcher {
            stop_flag,
            killers: [[None; 2]; MAX_PLY],
        }
    }

    fn negamax(
        &mut self,
        board: &mut Board,
        depth: i32,
        ply: usize,
        mut alpha: i32,
        beta: i32,
    ) -> i32 {
        if self.stop_flag.load(Ordering::Relaxed) {
            return 0;
        }

        if board.get_halfmove_clock() >= 100 {
            return 0;
        }

        if ply >= MAX_PLY {
            return board.get_board_evaluation();
        }

        if depth == 0 {
            return self.quiescence(board, alpha, beta);
        }

        let color = board.get_current_player_color();
        let mut legal_moves = gen_legal_moves(&board);
        if legal_moves.len() == 0 {
            return if rule::is_in_check(&board, color) {
                i32::MIN + 1
            } else {
                0
            };
        }
        legal_moves.rescore_with_killers(&self.killers[ply]);
        legal_moves.sort();

        let mut best_score = i32::MIN + 1;
        for mv in legal_moves {
            let state = board.make_move(mv);
            let score = -self.negamax(board, depth - 1, ply + 1, -beta, -alpha);
            board.unmake_move(mv, state);

            if score > best_score {
                best_score = score;
                if best_score > alpha {
                    alpha = best_score;
                }
            }
            if score >= beta {
                if mv.captured().is_none() && self.killers[ply][0] != Some(mv) {
                    self.killers[ply][1] = self.killers[ply][0];
                    self.killers[ply][0] = Some(mv);
                }
                return best_score;
            }
        }

        best_score
    }

    fn quiescence(&mut self, board: &mut Board, mut alpha: i32, beta: i32) -> i32 {
        if self.stop_flag.load(Ordering::Relaxed) {
            return 0;
        }

        let mut best_score = board.get_board_evaluation();
        if best_score >= beta {
            return best_score;
        }
        if alpha < best_score {
            alpha = best_score;
        }

        for mv in gen_legal_captures(&board) {
            // Delta pruning : si même en gagnant la pièce capturée (+ marge) on
            // ne remonte pas jusqu'à alpha, la branche est sans espoir. On garde
            // les promotions qui changent la valeur de la pièce promue.
            if mv.promotion().is_none() {
                if let Some(captured) = mv.captured() {
                    if best_score + captured.value() + DELTA_MARGIN < alpha {
                        continue;
                    }
                }
            }

            let state = board.make_move(mv);
            let score = -self.quiescence(board, -beta, -alpha);
            board.unmake_move(mv, state);

            if score >= beta {
                return score;
            }
            if score > best_score {
                best_score = score;
            }
            if score > alpha {
                alpha = score;
            }
        }

        best_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::board::Board;
    use std::sync::atomic::AtomicBool;

    fn eval(fen: &str, depth: i32) -> i32 {
        let mut searcher = Searcher::new(Arc::new(AtomicBool::new(false)));
        let mut board = Board::from_fen(fen).unwrap();
        searcher.negamax(&mut board, depth, 0, i32::MIN + 1, i32::MAX)
    }

    #[test]
    fn balanced_position_scores_zero_at_depth_0() {
        // Position avec matériel égal, éval doit être 0
        assert_eq!(eval("k7/8/8/8/8/8/8/K7 w - - 0 1", 0), 0);
    }

    #[test]
    fn captures_free_rook() {
        // Position équilibrée (tour vs tour). Blanc peut capturer la tour noire en g1.
        // Après Rxg1 : blanc K+R, noir K → score = +500
        assert!(eval("k7/8/8/8/8/8/8/KR4r1 w - - 0 1", 1) >= 500);
    }

    #[test]
    fn mate_in_one_is_max_score() {
        // Blanc joue Tb8# : pion a7 couvre b8, Ka6 couvre a7 et b7 → mat
        assert_eq!(eval("k7/P7/KR6/8/8/8/8/8 w - - 0 1", 2), i32::MAX);
    }

    #[test]
    fn stalemate_scores_zero() {
        // Noir à jouer, roi en a8, aucun coup légal mais pas en échec → nulle
        // Ka6 et Qb6 piègent le roi sans l'attaquer directement
        assert_eq!(eval("k7/8/KQ6/8/8/8/8/8 b - - 0 1", 1), 0);
    }

    #[test]
    fn fifty_move_rule_scores_zero() {
        assert_eq!(eval("k7/P7/KR6/8/8/8/8/8 w - - 100 1", 2), 0);
    }

    // --- Tactiques ---

    #[test]
    fn mate_in_two() {
        // Blanc joue Rb8+ Ka7, puis Rb7# — mat en 2 coups
        // Roi noir a8, tour b1, roi blanc c6
        let score = eval("k7/8/2K5/8/8/8/8/1R6 w - - 0 1", 4);
        assert_eq!(score, i32::MAX);
    }

    #[test]
    fn avoids_losing_piece() {
        // Tour blanche en h1 attaquée par la tour noire en h8, roi blanc loin
        // À depth 2 le searcher doit voir la perte et trouver une réponse qui ne perd pas la tour
        let score = eval("7r/8/8/8/8/8/8/k5RK w - - 0 1", 2);
        assert!(
            score > -500,
            "le searcher accepte de perdre la tour sans raison"
        );
    }

    #[test]
    fn winning_promotion_preferred() {
        // Pion blanc en a7, roi noir loin — promotion en dame est gagnante
        let score = eval("6k1/P7/8/8/8/8/8/6K1 w - - 0 1", 2);
        assert!(score > 0, "la promotion gagnante n'est pas choisie");
    }

    // --- Régression de score ---

    #[test]
    fn starting_position_is_symmetric() {
        // Position de départ : matériel et structure symétriques → score nul
        let score = eval(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            0,
        );
        assert_eq!(score, 0);
    }

    #[test]
    fn rook_up_is_positive() {
        // Blanc a une tour de plus → score positif à depth 0
        let score = eval("k7/8/8/8/8/8/8/KR6 w - - 0 1", 0);
        assert!(score > 0, "une tour d'avantage doit scorer positivement");
    }

    #[test]
    fn rook_down_is_negative() {
        // Noir a une tour de plus → score négatif pour blanc à depth 0
        let score = eval("kr6/8/8/8/8/8/8/K7 w - - 0 1", 0);
        assert!(score < 0, "une tour de moins doit scorer négativement");
    }
}
