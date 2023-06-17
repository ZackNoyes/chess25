
use wasm_bindgen::prelude::*;
use super::my_board::{MyBoard, MySquare, Status};
use chess::{BoardBuilder, ChessMove, Color, ALL_PIECES};
use js_sys::{Array, JsString};

#[wasm_bindgen]
impl MyBoard {
    pub fn initial_board() -> MyBoard {
        crate::utils::set_panic_hook();
        MyBoard {
            bb: BoardBuilder::default(),
            dead_moves: 0,
            status: Status::InProgress
        }
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

    pub fn js_get_side_to_move(&self) -> JsString {
        if self.bb.get_side_to_move().to_index() == 0 { "white".into() }
        else { "black".into() }
    }

    pub fn js_switch_side_to_move(&mut self) {
        let opp = if self.bb.get_side_to_move() == Color::White { Color::Black }
        else { Color::White };
        self.bb.side_to_move(opp);
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