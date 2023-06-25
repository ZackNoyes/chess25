
mod greedy;
mod minimax;
pub mod alphabeta;

mod evaluator;
pub mod proportion_count;

mod position_table;

use chess::{ChessMove, Color};
use crate::Score;
use crate::logger::Logger;
use crate::my_board::MyBoard;
use evaluator::StaticEvaluator;

pub trait Engine {
    fn default(static_evaluator: impl StaticEvaluator + 'static) -> Self where Self: Sized;
    fn evaluate(&mut self, board: &MyBoard) -> Score;

    fn get_move(&mut self, board: &MyBoard) -> ChessMove {

        let move_evaluations = board.all_moves().into_iter().map(|mv| {
            let (bonus_board, no_bonus_board) = self.next_boards(board, mv, true);
            // Assumes the chance of bonus and chance of no bonus
            let evaluation = self.evaluate(&bonus_board) * crate::bonus_chance()
                + self.evaluate(&no_bonus_board) * crate::no_bonus_chance();
            (mv, evaluation)
        });

        // This can be made more efficient, but this helps with debugging
        // The inefficiency is only at the top layer

        let mut move_evaluations: Vec<_> = move_evaluations.collect();
        move_evaluations.sort_by(|(_, a), (_, b)|
            if board.get_side_to_move() == Color::White {
                b.partial_cmp(a).unwrap()
            } else {
                a.partial_cmp(b).unwrap()
            }
        );

        self.log_info();

        let mut log_string = String::from("Top three moves considered: \n");

        for (i, move_eval) in move_evaluations.iter().take(3).enumerate() {
            log_string.push_str(
                &format!(
                    "{}: {} to {} with score {}\n",
                    i,
                    move_eval.0.get_source(),
                    move_eval.0.get_dest(),
                    move_eval.1
                )
            );
        }

        self.get_logger().log(5, &log_string);

        move_evaluations[0].0
    }

    /// Can be implemented to have certain information logged when a
    /// move is chosen.
    fn log_info(&self) {}

    /// Generate both the bonus and no bonus boards for a move. If `checked` is
    /// true, then `apply_bonus` will be called, but otherwise
    /// `apply_bonus_unchecked` will be called, which doesn't check for draws.
    fn next_boards(&self, board: &MyBoard, mv: ChessMove, checked: bool) -> (MyBoard, MyBoard) {
        let mut new_board = *board; new_board.apply_move_unchecked(mv);
        let mut bonus_board = new_board;
        let mut no_bonus_board = new_board;
        if checked {
            bonus_board.apply_bonus(true);
            no_bonus_board.apply_bonus(false);
        } else {
            bonus_board.apply_bonus_unchecked(true);
            no_bonus_board.apply_bonus_unchecked(false);
        }
        (bonus_board, no_bonus_board)
    }

    fn get_logger(&self) -> &Logger;
}

#[allow(dead_code)]
pub fn default_engine() -> impl Engine {
    alphabeta::AlphaBeta::default(proportion_count::ProportionCount::default())
}