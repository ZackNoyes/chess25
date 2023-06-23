
use chess::{Color::*, ChessMove};

use crate::{Score, ONE, ZERO, DELTA};
use crate::my_board::{MyBoard, Status};

use super::Engine;
use super::evaluator::StaticEvaluator;
use super::position_table::PositionTable;

pub struct AlphaBeta {
    static_evaluator: Box<dyn StaticEvaluator>,
    lookahead: u8,
    position_table: PositionTable<Bounds>,
}

/// Bounds for the possible evaluations for a move. The bounds are exclusive
/// on both sides. It should always be true that 0 <= min, max <= 1. It is not
/// necessarily true that min < max. To represent a min bound of 0 or a max
/// bound of 1 (i.e. no bound), `None` is used.
#[derive(Clone, Copy)]
pub struct Bounds {
    pub min: Option<Score>,
    pub max: Option<Score>,
}

impl Bounds {
    fn widest() -> Bounds {
        Bounds { min: None, max: None }
    }
    fn score_too_low(self, score: Score) -> bool {
        if let Some(min) = self.min { score <= min } else { false }
    }
    fn score_too_high(self, score: Score) -> bool {
        if let Some(max) = self.max { max <= score } else { false }
    }
    fn contains(self, score: Score) -> bool {
        !self.score_too_low(score) && !self.score_too_high(score)
    }
    fn min_decreased_by(self, amount: Score) -> Self {
        Bounds {
            min: if let Some(min) = self.min {
                min.checked_sub(amount)
            } else { None },
            max: self.max,
        }
    }
    /// Divides the bounds by the given amount. Slightly confusingly, I called
    /// this `expanded` because it is called with arguments less than 1.
    fn expanded(self, amount: Score) -> Self {
        assert!(amount > ZERO, "amount must be positive");
        assert!(amount < ONE, "amount must be less than 1");
        Bounds {
            min: if let Some(min) = self.min {
                let new_min = min.checked_div(amount)
                    .expect("expanding min should not overflow");
                assert!(new_min <= ONE, "new min should be <= 1");
                Some(new_min)
            } else { None },
            max: if let Some(max) = self.max {
                if let Some(new_max) = max.checked_div(amount) {
                    if new_max > ONE { None } else { Some(new_max) }
                } else { None }
            } else { None },
        }
    }
    fn both_decreased_by(self, amount: Score) -> Self {
        Bounds {
            min: if let Some(min) = self.min {
                min.checked_sub(amount)
            } else { None },
            max: if let Some(max) = self.max {
                Some(max.checked_sub(amount)
                    .expect("decreasing max should not overflow"))
            } else {
                Some(ONE - amount + DELTA)
                // we add DELTA since the bounds are inclusive. In practice this
                // just gets expanded and then becomes None.
            },
        }
    }
    fn valid(self) -> bool {
        if let Some(max) = self.max {
            max <= ONE &&
            if let Some(min) = self.min {
                min < max
            } else { true }
        } else { true }
    }
    fn update_min(&mut self, score: Score) {
        if let Some(min) = self.min {
            if score > min { self.min = Some(score); }
        } else { self.min = Some(score); }
    }
    fn update_max(&mut self, score: Score) {
        if let Some(max) = self.max {
            if score < max { self.max = Some(score); }
        } else { self.max = Some(score); }
    }
}

use SearchResult::*;
pub enum SearchResult {
    /// A score, optionally with a move that leads to that score.
    /// Most of the time, the move will be `None`, but it will be `Some` at the top
    /// level of the search tree.
    Result(Score, Option<ChessMove>), // TODO: space-optimise with the option
    /// The evaluation of the score is lower than the lower bound
    Low,
    /// The evaluation of the score is higher than the upper bound
    High,
}

impl AlphaBeta {

    pub fn new(static_evaluator: impl StaticEvaluator + 'static, lookahead: u8) -> Self {
        assert!(lookahead > 0, "lookahead must be positive");
        AlphaBeta {
            static_evaluator: Box::new(static_evaluator),
            lookahead,
            position_table: PositionTable::new(),
        }
    }

    /// Gets the best move for the current player, along with its score.
    /// 
    /// This function takes in `bounds` to search for the move within.
    /// If the search determines that the evaluation is outside the bounds,
    /// then a `Pruned` result is returned. Otherwise, a `Result` is
    /// returned. In this case, the result's `score` is guaranteed to be within
    /// the bounds.
    /// 
    /// The `Result` will have a `None` move if `depth` is 0, otherwise it will
    /// contain the move that led to that score.
    fn get_scored_best_move(&mut self, board: &MyBoard, bounds: Bounds, depth: u8) -> SearchResult {
        assert!(bounds.valid());
        let mut bounds = bounds;

        // TODO: Check Position Table

        if depth == 0 || !matches!(board.get_status(), Status::InProgress) {
            let evaluation = self.static_evaluator.evaluate(board);
            
            // TODO: Insert into Position Table

            return
                if bounds.score_too_low(evaluation) { Low }
                else if bounds.score_too_high(evaluation) { High }
                else { Result(evaluation, None) }
        }

        let is_maxing = matches!(board.get_side_to_move(), White);
        let mut best_result = None;

        for mv in board.all_moves() { // TODO: order moves
            
            let (b_board, nb_board) = self.next_boards(board, mv, depth != 1);

            // Calculate the implied bounds on the no-bonus branch, assuming
            // a worst-case scenario for the bonus branch at both sides of the
            // bound.
            let nb_bounds = bounds
                .min_decreased_by(crate::bonus_chance())
                .expanded(crate::no_bonus_chance());
            
            let nb_result = self.get_scored_best_move(&nb_board, nb_bounds, depth - 1);
            
            // Determine a probability weighted score for this move
            let score = if let Result(nb_score, _) = nb_result {
                let b_bounds = bounds
                    .both_decreased_by(nb_score * crate::no_bonus_chance())
                    .expanded(crate::bonus_chance());
                let b_result = self.get_scored_best_move(&b_board, b_bounds, depth - 1);
                if let Result(b_score, _) = b_result {
                    b_score * crate::bonus_chance()
                    + nb_score * crate::no_bonus_chance()
                } else {
                    if is_maxing == matches!(b_result, Low) { continue; }
                    else { return if is_maxing { High } else { Low } }
                }
            } else {
                if is_maxing == matches!(nb_result, Low) { continue; }
                else { return if is_maxing { High } else { Low } }
            };

            assert!(bounds.contains(score), "bounds should contain score \
                because of the constructed bounds on nb_score and b_score");

            // Update the bounds with this new result
            if is_maxing { bounds.update_min(score); }
            else { bounds.update_max(score); }

            // Update the best result found so far
            best_result = match best_result {
                None => Some((score, mv)),
                Some((best_score, _)) if
                    (is_maxing && score > best_score)
                    || (!is_maxing && score < best_score)
                => {
                    Some((score, mv))
                }
                _ => best_result,
            };

        }

        // TODO: Insert into Position Table

        if let Some((score, mv)) = best_result {
            Result(score, Some(mv))
        } else {
            return if is_maxing { Low } else { High };
        }
    }

}

impl Engine for AlphaBeta {

    fn default(static_evaluator: impl StaticEvaluator + 'static) -> Self {
        AlphaBeta::new(static_evaluator, 4)
    }

    fn evaluate(&mut self, board: &MyBoard) -> Score {
        match self.get_scored_best_move(board, Bounds::widest(), self.lookahead) {
            Result(score, _) => score,
            _ => panic!("pruning should not happen with the widest bounds"),
        }
    }

    fn get_move(&mut self, board: &MyBoard) -> ChessMove { // TODO: add more debug info
        let mv = match
            self.get_scored_best_move(board, Bounds::widest(), self.lookahead)
        {
            Result(_, mv) => mv.expect("move should be returned at top level"),
            _ => panic!("pruning should not happen with the widest bounds"),
        };
        self.log_info();
        mv
    }

    fn log_info(&mut self) {
        web_sys::console::log_1(&self.position_table.info().into());
        self.position_table.reset_debug_info();
    }

}
