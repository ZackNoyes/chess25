
mod evaluator;
mod proportion_count;

pub use evaluator::Evaluator;
pub use proportion_count::ProportionCount as DefaultEvaluator;

struct Engine {
  evaluator: Box<dyn Evaluator>,
}

