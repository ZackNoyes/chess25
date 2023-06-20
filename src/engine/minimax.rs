
use chess::Color;
use crate::my_board::{MyBoard, Status};
use super::{Engine, StaticEvaluator};

pub struct Minimax {
    static_evaluator: Box<dyn StaticEvaluator>,
    lookahead: u8,
}

impl Minimax {

    pub fn new(static_evaluator: impl StaticEvaluator + 'static, lookahead: u8) -> Self {
        Minimax {
            static_evaluator: Box::new(static_evaluator),
            lookahead,
        }
    }

    fn evaluate_with_cutoff(&mut self, board: &MyBoard, cutoff: u8) -> f64 {
        
        if cutoff == 0 || !matches!(board.get_status(), Status::InProgress) {
            return self.static_evaluator.evaluate(board);
        }

        let scores = board.all_moves().into_iter().map(|mv| {

            let mut new_board = *board; new_board.apply_move(mv);
            let mut bonus_board = new_board; bonus_board.apply_bonus(true);
            let mut no_bonus_board = new_board; no_bonus_board.apply_bonus(false);
            
            0.25 * self.evaluate_with_cutoff(&bonus_board, cutoff - 1)
            + 0.75 * self.evaluate_with_cutoff(&no_bonus_board, cutoff - 1)
        });

        if matches!(board.get_bb().get_side_to_move(), Color::White) {
            scores.max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
        } else {
            scores.min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
        }
    }

}

impl Engine for Minimax {
    fn default(static_evaluator: impl StaticEvaluator + 'static) -> Self {
        Minimax {
            static_evaluator: Box::new(static_evaluator),
            lookahead: 2,
        }
    }

    fn evaluate(&mut self, board: &MyBoard) -> f64 {
        self.evaluate_with_cutoff(&board, self.lookahead - 1)
    }
}
