
use wasm_bindgen::prelude::*;
use crate::{my_board::{MyBoard, Status}, engine::Engine};
use chess::{ChessMove, Color, ALL_PIECES, Square};
use js_sys::{Array, JsString};

#[wasm_bindgen]
pub struct JSInterface {
    board: MyBoard,
    engine_black: Box<dyn Engine>,
    engine_white: Box<dyn Engine>,
}

#[wasm_bindgen]
impl JSInterface {

    pub fn js_initial_interface(white_starts: bool) -> Self {
        let weights = crate::engine::feature_eval::Weights {
            pieces: [[1.0, 3.0, 3.0, 5.0, 9.0, 0.0], [-1.0, -3.0, -3.0, -5.0, -9.0, 0.0]],
            king_danger: [-1.0, 1.0],
            pawn_advancement: [1.0, -1.0],
            side_to_move: 3.0,
        };
        crate::utils::set_panic_hook();
        JSInterface {
            board: MyBoard::initial_board(
                if white_starts { Color::White } else { Color::Black }
            ),
            engine_black: Box::new(
                crate::engine::alphabeta::AlphaBeta::new(
                    crate::engine::feature_eval::FeatureEval::new(weights, 22.0),
                    4, true, 10
                )
            ),
            engine_white: Box::new(
                crate::engine::alphabeta::AlphaBeta::new(
                    crate::engine::proportion_count::ProportionCount::default(),
                    4, false, 10
                )
            )
        }
    }

    pub fn js_piece(&self, file: usize, rank: usize) -> Option<JsString> {
        let square = make_square(file, rank);
        match self.board[square] {
            Some((p, c)) => Some(p.to_string(c).into()),
            _ => None
        }
    }

    pub fn js_piece_color(&self, file: usize, rank: usize) -> JsString {
        let square = make_square(file, rank);
        match self.board[square] {
            Some((_, Color::White)) => "white".into(),
            Some((_, Color::Black)) => "black".into(),
            _ => "empty".into()
        }
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
    pub fn js_check_move(&self,
        from_file: usize, from_rank: usize, to_file: usize, to_rank: usize
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

    pub fn js_apply_move(&mut self,
        from_file: usize, from_rank: usize, to_file: usize, to_rank: usize,
        promotion: Option<usize>
    ) {
        let from = make_square(from_file, from_rank);
        let to = make_square(to_file, to_rank);
        let m = ChessMove::new(from, to, promotion.map(|i| ALL_PIECES[i]));
        self.board.apply_move(m);
    }

    pub fn js_apply_bonus(&mut self, is_bonus: bool) {
        self.board.apply_bonus(is_bonus);
    }

    pub fn js_get_side_to_move(&self) -> JsString {
        if self.board.get_side_to_move().to_index() == 0 { "white".into() }
        else { "black".into() }
    }

    pub fn js_status(&self) -> JsString {
        self.board.get_status().into()
    }

    pub fn js_get_engine_move(&mut self) -> Array {
        move_to_array(
            match self.board.get_side_to_move() {
                Color::White => self.engine_white.get_move(&self.board),
                Color::Black => self.engine_black.get_move(&self.board)
            }
        )
    }
}

impl From<Status> for JsString {
    fn from(r: Status) -> JsString {
        match r {
            Status::InProgress => "in progress".into(),
            Status::Win(Color::White) => "white".into(),
            Status::Win(Color::Black) => "black".into(),
            Status::Draw => "draw".into()
        }
    }
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
    Square::make_square(
        chess::Rank::from_index(rank),
        chess::File::from_index(file)
    )
}