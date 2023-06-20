
mod greedy;

mod evaluator;
mod proportion_count;

use chess::ChessMove;
use crate::my_board::MyBoard;
use evaluator::StaticEvaluator;

pub trait Engine {
    fn new(static_evaluator: impl StaticEvaluator + 'static) -> Self;
    fn evaluate(&mut self, board: &MyBoard) -> f64;

    fn get_move(&mut self, board: &MyBoard) -> ChessMove {
        board.all_moves().into_iter().map(|mv| {
            let mut new_board = *board;
            new_board.apply_move(mv);
            (mv, self.evaluate(&new_board))
        }).max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0
    }
}

pub fn default_engine() -> impl Engine {
    greedy::Greedy::new(proportion_count::ProportionCount::default())
}