use crate::Score;
use chess::ChessMove;

#[derive(Debug)]
pub enum SearchResult {
    /// A score, optionally with a move that leads to that score.
    /// Most of the time, the move will be `None`, but it will be `Some` at the top
    /// level of the search tree.
    Result(Score, Option<ChessMove>),
    /// The evaluation of the score is lower than the lower bound
    Low,
    /// The evaluation of the score is higher than the upper bound
    High,
}