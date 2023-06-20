
use wasm_bindgen::prelude::*;
use crate::my_board::{MyBoard, MySquare, Status};
use chess::{ChessMove, Color, ALL_PIECES};
use js_sys::{Array, JsString};

#[wasm_bindgen]
impl MyBoard {

    pub fn js_initial_board(white_starts: bool) -> MyBoard {
        crate::utils::set_panic_hook();
        MyBoard::initial_board(
            if white_starts { Color::White } else { Color::Black }
        )
    }

    pub fn js_piece(&self, file: usize, rank: usize) -> Option<JsString> {
        let square = MySquare::new(file, rank);
        match self[square] {
            Some((p, c)) => Some(p.to_string(c).into()),
            _ => None
        }
    }

    pub fn js_piece_color(&self, file: usize, rank: usize) -> JsString {
        let square = MySquare::new(file, rank);
        match self[square] {
            Some((_, Color::White)) => "white".into(),
            Some((_, Color::Black)) => "black".into(),
            _ => "empty".into()
        }
    }

    pub fn js_moves_from(&self, file: usize, rank: usize) -> Array {
        let square = MySquare::new(file, rank);
        let moves = self.moves_from(square);
        let js_moves = Array::new();
        for m in moves {
            let js_move = Array::new();
            js_move.push(&m.get_dest().get_file().to_index().into());
            js_move.push(&m.get_dest().get_rank().to_index().into());
            js_move.push(&
                if let Some(p) = m.get_promotion() { Some(p.to_index()) }
                else { None }
                .into()
            );
            js_moves.push(&js_move);
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
        let from = MySquare::new(from_file, from_rank);
        let to = MySquare::new(to_file, to_rank);
        let m = ChessMove::new(from.0, to.0, None);
        let mp = ChessMove::new(from.0, to.0, Some(ALL_PIECES[1]));
        if self.moves_from(from).contains(&m) {
            Some(false)
        } else if self.moves_from(from).contains(&mp) {
            Some(true)
        } else {
            None
        }
    }

    pub fn js_apply_move(&mut self,
        from_file: usize, from_rank: usize, to_file: usize, to_rank: usize,
        promotion: Option<usize>
    ) {
        let from = MySquare::new(from_file, from_rank);
        let to = MySquare::new(to_file, to_rank);
        let m = ChessMove::new(from.0, to.0, promotion.map(|i| ALL_PIECES[i]));
        self.apply_move(m);
    }

    pub fn js_apply_bonus(&mut self, is_bonus: bool) {
        self.apply_bonus(is_bonus);
    }

    pub fn js_get_side_to_move(&self) -> JsString {
        if self.bb.get_side_to_move().to_index() == 0 { "white".into() }
        else { "black".into() }
    }

    pub fn js_status(&self) -> JsString {
        self.status.into()
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