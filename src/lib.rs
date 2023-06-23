use fixed::{FixedU32, types::extra::U31};

mod utils;
mod my_board;
mod js_interface;
mod engine;
mod zobrist;

type Score = FixedU32<U31>;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;