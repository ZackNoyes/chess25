use fixed::{FixedU32, types::extra::U31};

mod utils;
mod my_board;
mod js_interface;
mod engine;
mod zobrist;

type Score = FixedU32<U31>;
pub const ONE: Score = Score::ONE;
pub const ZERO: Score = Score::ZERO;
pub const DELTA: Score = Score::DELTA;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[inline]
pub fn bonus_chance() -> Score {
    ONE / 4
}

#[inline]
pub fn no_bonus_chance() -> Score {
    ONE - bonus_chance()
}