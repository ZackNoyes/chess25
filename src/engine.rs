
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
            if matches!(board.get_board().get_side_to_move(), Color::White) { 1.0 }
            else { -1.0 };

        let move_evaluations = board.all_moves().into_iter().map(|mv| {
            let mut new_board = *board; new_board.apply_move(mv);
            let mut bonus_board = new_board; bonus_board.apply_bonus(true);
            let mut no_bonus_board = new_board; no_bonus_board.apply_bonus(false);
            let evaluation =
                CHANCE_OF_BONUS * self.evaluate(&bonus_board)
                + CHANCE_OF_NO_BONUS * self.evaluate(&no_bonus_board);
            (mv, multiplier * evaluation)
        });

        // This can be made more efficient, but this helps with debugging
        // The inefficiency is only at the top layer

        let mut move_evaluations: Vec<_> = move_evaluations.collect();
        move_evaluations.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());

        let mut log_string = String::from("Top three moves considered: \n");

        for i in 0..3 {
            log_string.push_str(
                &format!(
                    "{}: {} to {} with score {}\n",
                    i,
                    move_evaluations[i].0.get_source(),
                    move_evaluations[i].0.get_dest(),
                    -move_evaluations[i].1
                )
            );
        }

        web_sys::console::log_1(&log_string.into());

        move_evaluations[0].0
    }
}

pub fn default_engine() -> impl Engine {
    minimax::Minimax::new(proportion_count::ProportionCount::default(), 4)
}