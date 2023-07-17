use chess::{ChessMove, Color::*, Square, ALL_COLORS, ALL_PIECES};
use js_sys::{Array, JsString};
use wasm_bindgen::prelude::*;

use crate::{
    engine::Engine,
    my_board::{MyBoard, Status},
};

#[wasm_bindgen]
pub struct JSInterface {
    board: MyBoard,
    engine_black: Box<dyn Engine>,
    engine_white: Box<dyn Engine>,
    board_history: Vec<MyBoard>,
    move_history: Vec<ChessMove>,
}

#[wasm_bindgen]
impl JSInterface {
    pub fn js_initial_interface(white_starts: bool) -> Self {
        crate::utils::set_panic_hook();
        // This will crash the code on a wasm target and refresh the page after
        // 02/09/2023 So that it's harder for people to steal my WASM
        explode(1693663199000);
        let weights = crate::engine::feature_eval::Weights {
            pieces: [[1.0, 3.0, 3.0, 5.0, 9.0, 0.0], [
                -1.0, -3.0, -3.0, -5.0, -9.0, 0.0,
            ]],
            king_danger: [-0.5, 0.5],
            pawn_advancement: [0.5, -0.5],
            side_to_move: 3.0,
        };
        JSInterface {
            board: MyBoard::initial_board(if white_starts { White } else { Black }),
            engine_black: Box::new(crate::engine::alphabeta::AlphaBeta::new(
                crate::engine::feature_eval::FeatureEval::new(weights, 15.0),
                10,
                true,
                false,
                3,
                1000,
            )),
            engine_white: Box::new(crate::engine::alphabeta::AlphaBeta::new(
                crate::engine::feature_eval::FeatureEval::new(weights, 15.0),
                10,
                true,
                false,
                3,
                1000,
            )),
            board_history: Vec::new(),
            move_history: Vec::new(),
        }
    }

    pub fn js_piece(&self, file: usize, rank: usize) -> Option<JsString> {
        let square = make_square(file, rank);
        match self.board[square] {
            Some((p, c)) => Some(p.to_string(c).into()),
            _ => None,
        }
    }

    pub fn js_history_piece(&self, file: usize, rank: usize, index: usize) -> Option<JsString> {
        let square = make_square(file, rank);
        match self.board_history[index][square] {
            Some((p, c)) => Some(p.to_string(c).into()),
            _ => None,
        }
    }

    pub fn js_history_was_hot(&self, file: usize, rank: usize, index: usize) -> bool {
        let square = make_square(file, rank);
        self.move_history[index].get_source() == square
            || self.move_history[index].get_dest() == square
    }

    pub fn js_piece_color(&self, file: usize, rank: usize) -> JsString {
        let square = make_square(file, rank);
        match self.board[square] {
            Some((_, White)) => "white".into(),
            Some((_, Black)) => "black".into(),
            _ => "empty".into(),
        }
    }

    pub fn js_checked_squares(&self) -> Array {
        let checked_kings = Array::new();
        if !self.board.get_status().is_in_progress() {
            return checked_kings;
        }
        for c in ALL_COLORS {
            if self.board.in_check(c) {
                checked_kings.push(&square_to_array(
                    self.board
                        .king_square(c)
                        .expect("there should be kings on the board"),
                ));
            }
        }
        checked_kings
    }

    pub fn js_moves_from(&self, file: usize, rank: usize) -> Array {
        let square = make_square(file, rank);
        let moves = self.board.moves_from(square);
        let js_moves = Array::new();
        for m in moves {
            js_moves.push(&move_to_array(m));
        }
        js_moves
    }

    /// Returns:
    /// - `Some(true)` if the move is legal and has a promotion
    /// - `Some(false)` if the move is legal and does not have a promotion
    /// - `None` if the move is illegal
    pub fn js_check_move(
        &self, from_file: usize, from_rank: usize, to_file: usize, to_rank: usize,
    ) -> Option<bool> {
        let from = make_square(from_file, from_rank);
        let to = make_square(to_file, to_rank);
        let m = ChessMove::new(from, to, None);
        let mp = ChessMove::new(from, to, Some(ALL_PIECES[1]));
        if self.board.moves_from(from).contains(&m) {
            Some(false)
        } else if self.board.moves_from(from).contains(&mp) {
            Some(true)
        } else {
            None
        }
    }

    pub fn js_apply_move(
        &mut self, from_file: usize, from_rank: usize, to_file: usize, to_rank: usize,
        promotion: Option<usize>,
    ) {
        let from = make_square(from_file, from_rank);
        let to = make_square(to_file, to_rank);
        let m = ChessMove::new(from, to, promotion.map(|i| ALL_PIECES[i]));
        self.board.apply_move(m);
        self.board_history.push(self.board);
        self.move_history.push(m);
    }

    pub fn js_apply_bonus(&mut self, is_bonus: bool) { self.board.apply_bonus(is_bonus); }

    pub fn js_get_side_to_move(&self) -> JsString {
        if self.board.get_side_to_move().to_index() == 0 {
            "white".into()
        } else {
            "black".into()
        }
    }

    pub fn js_status(&self) -> JsString { self.board.get_status().into() }

    pub fn js_get_engine_move(&mut self) -> Array {
        move_to_array(match self.board.get_side_to_move() {
            White => self.engine_white.get_move(&self.board),
            Black => self.engine_black.get_move(&self.board),
        })
    }
}

impl From<Status> for JsString {
    fn from(r: Status) -> JsString {
        match r {
            Status::InProgress => "in progress".into(),
            Status::Win(White) => "white".into(),
            Status::Win(Black) => "black".into(),
            Status::Draw => "draw".into(),
        }
    }
}

fn square_to_array(s: Square) -> Array {
    let js_square = Array::new();
    js_square.push(&s.get_file().to_index().into());
    js_square.push(&s.get_rank().to_index().into());
    js_square
}

fn move_to_array(m: ChessMove) -> Array {
    let js_move = Array::new();
    js_move.push(&m.get_source().get_file().to_index().into());
    js_move.push(&m.get_source().get_rank().to_index().into());
    js_move.push(&m.get_dest().get_file().to_index().into());
    js_move.push(&m.get_dest().get_rank().to_index().into());
    js_move.push(&m.get_promotion().map(|p| p.to_index()).into());
    js_move
}

fn make_square(file: usize, rank: usize) -> Square {
    Square::make_square(chess::Rank::from_index(rank), chess::File::from_index(file))
}

#[cfg(target_arch = "wasm32")]
fn explode(millis: u64) {
    if !(millis as f64 - js_sys::Date::now() > 0.0) {
        web_sys::window().unwrap().location().reload().unwrap();
        panic!("Something went wrong.")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn explode(_: u64) {}
