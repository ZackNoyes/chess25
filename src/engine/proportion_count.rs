use chess::Color;
use wasm_bindgen::prelude::wasm_bindgen;

use super::StaticEvaluator;
use crate::{my_board::MyBoard, Score};

const PIECE_VALUES: [u8; 6] = [1, 3, 3, 5, 9, 1];

#[wasm_bindgen]
#[derive(Default)]
pub struct ProportionCount;

impl StaticEvaluator for ProportionCount {
    fn evaluate(&self, board: &MyBoard) -> Score {
        if !board.get_status().is_in_progress() {
            return self.evaluate_terminal(board).unwrap();
        }

        let mut white_value = 0;
        let mut black_value = 0;

        for sq in board.get_white_pieces() {
            let Some((piece, Color::White)) = board[sq]
                else { panic!("White piece not found on square {:?}", sq); };
            let value = PIECE_VALUES[piece.to_index()];
            white_value += value;
        }

        for sq in board.get_black_pieces() {
            let Some((piece, Color::Black)) = board[sq]
                else { panic!("Black piece not found on square {:?}", sq); };
            let value = PIECE_VALUES[piece.to_index()];
            black_value += value;
        }

        let total_value = white_value + black_value;
        Score::from_num(white_value as f32 / total_value as f32)
    }
}
