
use std::collections::HashMap;
use chess::{Piece, Color};
use wasm_bindgen::prelude::wasm_bindgen;

use crate::my_board::{MyBoard, Status, MySquare};
use super::StaticEvaluator;

#[wasm_bindgen]
pub struct ProportionCount {
    piece_values: HashMap<Piece, u8>,
}

impl Default for ProportionCount {

    fn default() -> Self {
        let mut piece_values = HashMap::new();
        piece_values.insert(Piece::Pawn, 1);
        piece_values.insert(Piece::Knight, 3);
        piece_values.insert(Piece::Bishop, 3);
        piece_values.insert(Piece::Rook, 5);
        piece_values.insert(Piece::Queen, 9);
        piece_values.insert(Piece::King, 1);
        ProportionCount {
            piece_values
        }
    }

}

impl StaticEvaluator for ProportionCount {

    fn evaluate(&self, board: &MyBoard) -> f64 {
        
        if !matches!(board.get_status(), Status::InProgress) {
            return self.evaluate_terminal(board).unwrap();
        }

        let mut white_value = 0;
        let mut black_value = 0;
        
        for sq in MySquare::all_squares() {
            if let Some((piece, color)) = board[sq] {
                let value = self.piece_values[&piece];
                if matches!(color, Color::White) {
                    white_value += value;
                } else {
                    black_value += value;
                }
            }
        }

        let total_value = white_value + black_value;
        white_value as f64 / total_value as f64
    }

}