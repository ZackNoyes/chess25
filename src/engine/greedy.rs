
use crate::my_board::MyBoard;
use super::{Engine, StaticEvaluator};

pub struct Greedy {
    static_evaluator: Box<dyn StaticEvaluator>,
}

impl Engine for Greedy {
    fn new(static_evaluator: impl StaticEvaluator + 'static) -> Self {
        Greedy { static_evaluator: Box::new(static_evaluator) }
    }

    fn evaluate(&mut self, board: &MyBoard) -> f64 {
        self.static_evaluator.evaluate(board)
    }
}
