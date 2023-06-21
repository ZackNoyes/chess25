use crate::my_board::{MyBoard, Status};
use chess::Color;

pub trait StaticEvaluator {

    /// Evaluates a given game state represented by `board`.
    /// Returns a float between 0 and 1, which should be equal to
    /// `0 * P(B) + 0.5 * P(D) + 1 * P(W)`, where:
    /// - `P(B)` is the probability of black winning
    /// - `P(D)` is the probability of a draw
    /// - `P(W)` is the probability of white winning
    /// 
    /// That is, it should return the expected value of the position for white,
    /// given that the value of a win is 1 and the value of a draw is 0.5.
    fn evaluate(&self, board: &MyBoard) -> f64;

    /// Returns the evaluation of a terminal game state, or None if the game
    /// is still in progress.
    fn evaluate_terminal(&self, board: &MyBoard) -> Option<f64> {
        match board.get_status() {
            Status::InProgress => None,
            Status::Win(Color::Black) => Some(0.0),
            Status::Win(Color::White) => Some(1.0),
            Status::Draw => Some(0.5),
        }
    }

}
