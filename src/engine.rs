pub mod board;
pub mod movegen;
pub mod mv;
pub mod piece;
pub mod rule;
pub mod searcher;
pub mod square;

use core::time::Duration;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver},
};

use board::Board;
use piece::{Color, Piece};
use searcher::search;
use square::Square;
use mv::Move;

pub struct SquareMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>,
}

pub enum TimeControl {
    Depth(u32),
    MoveTime(u64),
    Clock { time_ms: u64, inc_ms: u64 },
    Infinite,
    Ponder,
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

impl SearchParams {
    pub fn time_control(&self, side: Color) -> TimeControl {
        if self.ponder {
            return TimeControl::Ponder;
        }
        if self.infinite {
            return TimeControl::Infinite;
        }
        if let Some(ms) = self.movetime {
            return TimeControl::MoveTime(ms);
        }
        if let Some(depth) = self.depth {
            return TimeControl::Depth(depth);
        }
        let (time, inc) = match side {
            Color::White => (self.wtime.unwrap_or(0), self.winc.unwrap_or(0)),
            Color::Black => (self.btime.unwrap_or(0), self.binc.unwrap_or(0)),
        };
        TimeControl::Clock {
            time_ms: time,
            inc_ms: inc,
        }
    }
}

pub trait ChessEngine {
    fn new_game(&mut self);
    fn set_position(&mut self, fen: Option<&str>, moves: &[SquareMove]);
    fn start_search(&mut self, params: SearchParams) -> Receiver<Option<Move>>;
    fn stop(&mut self);
    fn ponderhit(&mut self);
}

pub struct Engine {
    board: Board,
    stop_flag: Arc<AtomicBool>,
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            board: Board::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
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

    fn start_search(&mut self, params: SearchParams) -> Receiver<Option<Move>> {
        let (tx, rx) = mpsc::channel();
        match params.time_control(self.board.get_current_player_color()) {
            TimeControl::Clock { time_ms, inc_ms } => {
                let search_time = time_ms / 20 + inc_ms / 2;
                let stop_timer = Arc::clone(&self.stop_flag);
                let stop_search = Arc::clone(&self.stop_flag);
                self.stop_flag.store(false, Ordering::Relaxed);
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(search_time));
                    stop_timer.store(true, Ordering::Relaxed);
                });
                let board = self.board.clone();
                std::thread::spawn(move || {
                    search(board, stop_search, tx);
                });
            }
            _ => {
                println!("Only clock time control is supported");
                tx.send(None).ok();
            }
        }
        rx
    }

    fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    fn ponderhit(&mut self) {
        todo!()
    }
}

// let think_ms = time_ms / 20 + inc_ms / 2;
