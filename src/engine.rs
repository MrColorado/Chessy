pub mod board;
pub mod piece;
pub mod mv;
pub mod square;
pub mod rule;
pub mod movegen;

use board::Board;
use square::Square;
use piece::Piece;

pub struct SquareMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>,
}

pub struct SearchParams {
    pub depth: Option<u32>,
    pub movetime: Option<u64>,
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub infinite: bool,
    pub ponder: bool,
}

pub trait ChessEngine {
    fn new_game(&mut self);
    fn set_position(&mut self, fen: Option<&str>, moves: &[SquareMove]);
    fn start_search(&mut self, params: SearchParams);
    fn stop(&mut self);
    fn ponderhit(&mut self);
}

pub struct Engine {
    board: Board,
}

impl Engine {
    pub fn new() -> Engine {
        Engine { board: Board::new() }
    }
}

impl ChessEngine for Engine {
    fn new_game(&mut self) {
        self.board = Board::new();
    }

    fn set_position(&mut self, fen: Option<&str>, moves: &[SquareMove]) {
        self.board = match fen {
            Some(s) => Board::from_fen(s).expect("invalid FEN"),
            None => Board::new(),
        };
        for mv in moves {
            self.board
                .make_squares_move(mv.from, mv.to, mv.promotion)
                .expect("invalid move in position");
        }
    }

    fn start_search(&mut self, _params: SearchParams) {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }

    fn ponderhit(&mut self) {
        todo!()
    }
}
