
mod greedy;
mod minimax;

mod evaluator;
mod proportion_count;

use chess::{ChessMove, Color};
use crate::{CHANCE_OF_BONUS, CHANCE_OF_NO_BONUS};
use crate::my_board::MyBoard;
use evaluator::StaticEvaluator;

pub trait Engine {
    fn default(static_evaluator: impl StaticEvaluator + 'static) -> Self where Self: Sized;
    fn evaluate(&mut self, board: &MyBoard) -> f64;

    fn get_move(&mut self, board: &MyBoard) -> ChessMove {

        // for controlling whether we choose the maximum or the minimum
        let multiplier =
            if matches!(board.get_bb().get_side_to_move(), Color::White) { 1.0 }
            else { -1.0 };

        board.all_moves().into_iter().map(|mv| {
            let mut new_board = *board; new_board.apply_move(mv);
            let mut bonus_board = new_board; bonus_board.apply_bonus(true);
            let mut no_bonus_board = new_board; no_bonus_board.apply_bonus(false);
            let evaluation =
                CHANCE_OF_BONUS * self.evaluate(&bonus_board)
                + CHANCE_OF_NO_BONUS * self.evaluate(&no_bonus_board);
            (mv, multiplier * evaluation)
        }).max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0
    }
}

pub fn default_engine() -> impl Engine {
    minimax::Minimax::new(proportion_count::ProportionCount::default(), 2)
}