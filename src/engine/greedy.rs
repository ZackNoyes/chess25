
use crate::{my_board::MyBoard, Score, logger::Logger};
use super::{Engine, StaticEvaluator};

pub struct Greedy {
    static_evaluator: Box<dyn StaticEvaluator>,
    logger: Logger,
}

impl Engine for Greedy {
    fn default(static_evaluator: impl StaticEvaluator + 'static) -> Self {
        Greedy {
            static_evaluator: Box::new(static_evaluator),
            logger: Logger::new(0)
        }
    }

    fn evaluate(&mut self, board: &MyBoard) -> Score {
        self.static_evaluator.evaluate(board)
    }

    fn get_logger(&self) -> &Logger {
        &self.logger
    }
}
