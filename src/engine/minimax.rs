
use chess::Color;

use crate::Score;
use crate::my_board::{MyBoard, Status};
use super::position_table::{PositionTable};
use super::{Engine, StaticEvaluator};

pub struct Minimax {
    static_evaluator: Box<dyn StaticEvaluator>,
    lookahead: u8,
    position_table: PositionTable<Score>,
}

impl Minimax {

    pub fn new(static_evaluator: impl StaticEvaluator + 'static, lookahead: u8) -> Self {
        Minimax {
            static_evaluator: Box::new(static_evaluator),
            lookahead,
            position_table: PositionTable::new(),
        }
    }

    fn evaluate_with_cutoff(&mut self, board: &MyBoard, cutoff: u8) -> Score {

        if let Some(score) = self.position_table.get(board, cutoff) {
            return score;
        }

        if cutoff == 0 || !matches!(board.get_status(), Status::InProgress) {
            let evaluation = self.static_evaluator.evaluate(board);
            self.position_table.insert_both_colors(board, cutoff, evaluation);
            return evaluation;
        }

        let scores = board.all_moves().into_iter().map(|mv| {

            // At the last layer, we skip the draw check, since it's really
            // rare and also the most expensive part
            let (bonus_board, no_bonus_board) = self.next_boards(board, mv, cutoff != 1);
            
            // Assumes the chance of bonus and chance of no bonus
            self.evaluate_with_cutoff(&bonus_board, cutoff - 1) * crate::bonus_chance()
            + self.evaluate_with_cutoff(&no_bonus_board, cutoff - 1) * crate::no_bonus_chance()
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
        Minimax::new(static_evaluator, 4)
    }

    fn evaluate(&mut self, board: &MyBoard) -> Score {
        self.evaluate_with_cutoff(board, self.lookahead - 1)
    }

}
