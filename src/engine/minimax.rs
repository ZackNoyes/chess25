
use chess::Color;

use crate::{CHANCE_OF_BONUS, CHANCE_OF_NO_BONUS};
use crate::my_board::{MyBoard, Status};
use super::position_table::{PositionTable};
use super::{Engine, StaticEvaluator};

pub struct Minimax {
    static_evaluator: Box<dyn StaticEvaluator>,
    lookahead: u8,
    position_table: PositionTable,
}

impl Minimax {

    pub fn new(static_evaluator: impl StaticEvaluator + 'static, lookahead: u8) -> Self {
        Minimax {
            static_evaluator: Box::new(static_evaluator),
            lookahead,
            position_table: PositionTable::new(),
        }
    }

    fn evaluate_with_cutoff(&mut self, board: &MyBoard, cutoff: u8) -> f64 {

        if let Some(score) = self.position_table.get(board, cutoff) {
            return score;
        }

        if cutoff == 0 || !matches!(board.get_status(), Status::InProgress) {
            let evaluation = self.static_evaluator.evaluate(board);
            self.position_table.insert_both_colors(board, cutoff, evaluation);
            return evaluation;
        }

        let scores = board.all_moves().into_iter().map(|mv| {

            let mut new_board = *board; new_board.apply_move(mv);

            let mut bonus_board = new_board;
            let mut no_bonus_board = new_board;

            // At the last layer, we skip the draw check, since it's really
            // rare and also the most expensive part
            if cutoff == 1 {
                bonus_board.apply_bonus_unchecked(true);
                no_bonus_board.apply_bonus_unchecked(false);
            } else {
                bonus_board.apply_bonus(true);
                no_bonus_board.apply_bonus(false);
            }
            
            CHANCE_OF_BONUS * self.evaluate_with_cutoff(&bonus_board, cutoff - 1)
            + CHANCE_OF_NO_BONUS * self.evaluate_with_cutoff(&no_bonus_board, cutoff - 1)
        });

        let score = if matches!(board.get_side_to_move(), Color::White) {
            scores.max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
        } else {
            scores.min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
        };

        self.position_table.insert(board, cutoff, score);

        score
    }

}

impl Engine for Minimax {
    fn default(static_evaluator: impl StaticEvaluator + 'static) -> Self {
        Minimax::new(static_evaluator, 3)
    }

    fn evaluate(&mut self, board: &MyBoard) -> f64 {
        self.evaluate_with_cutoff(&board, self.lookahead - 1)
    }
}
