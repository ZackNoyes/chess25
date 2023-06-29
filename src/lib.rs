use fixed::{types::extra::U31, FixedU32};

mod deadline;
mod engine;
mod js_interface;
mod logger;
mod my_board;
mod utils;
mod zobrist;

pub type Score = FixedU32<U31>;
pub(crate) const ONE: Score = Score::ONE;
pub(crate) const ZERO: Score = Score::ZERO;
pub(crate) const DELTA: Score = Score::DELTA;

pub use engine::{
    alphabeta::AlphaBeta,
    feature_eval::{FeatureEval, Features, Weights},
    proportion_count::ProportionCount,
    Engine, StaticEvaluator,
};
pub use logger::Logger;
pub use my_board::{MyBoard, Status};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[inline]
pub fn bonus_chance() -> Score { ONE / 4 }

#[inline]
pub fn no_bonus_chance() -> Score { ONE - bonus_chance() }
