mod utils;
mod my_board;
mod js_interface;
mod engine;

const CHANCE_OF_BONUS: f64 = 0.25;
const CHANCE_OF_NO_BONUS: f64 = 1.0 - CHANCE_OF_BONUS;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;