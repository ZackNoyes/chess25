
use core::panic;

use chess::{Color::*, ChessMove};

use crate::{Score, ONE, ZERO, DELTA};
use crate::my_board::{MyBoard, Status};

use super::Engine;
use super::evaluator::StaticEvaluator;
use super::position_table::PositionTable;

pub struct AlphaBeta {
    static_evaluator: Box<dyn StaticEvaluator>,
    lookahead: u8,
    position_table: PositionTable<ScoreInfo>,
    // Debug info
    rounding_errors: u32,
    branch_info: Vec<BranchInfo>,
}

/// Bounds for the possible evaluations for a move. The bounds are exclusive
/// on both sides. It should always be true that 0 <= min, max <= 1. It is not
/// necessarily true that min < max. To represent a min bound of 0 or a max
/// bound of 1 (i.e. no bound), `None` is used.
#[derive(Clone, Copy, Debug)]
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
    fn info_too_low(self, score_info: ScoreInfo) -> bool {
        if let Some(min) = self.min {
            if score_info.max <= min { return true; }
        }
        false
    }
    fn info_too_high(self, score_info: ScoreInfo) -> bool {
        if let Some(max) = self.max {
            if score_info.min >= max { return true; }
        }
        false
    }
}

/// Stores a pair of bounds for the score of a given position. Unlike `Bounds`,
/// the bounds are inclusive on both sides, so `ZERO` and `ONE` can be used for
/// the min and max bounds.
/// 
/// This is used in the position table to store the results of the search.
#[derive(Clone, Copy)]
struct ScoreInfo {
    min: Score,
    max: Score,
}
impl ScoreInfo {
    fn actual_score(self) -> Option<Score> {
        if self.min == self.max { Some(self.min) } else { None }
    }
    fn from_score(score: Score) -> Self {
        ScoreInfo { min: score, max: score }
    }
    fn from_min_score(min: Score) -> Self {
        ScoreInfo { min, max: ONE }
    }
    fn from_max_score(max: Score) -> Self {
        ScoreInfo { min: ZERO, max }
    }
}

use SearchResult::*;
#[derive(Debug)]
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

/// A way to record statistics about the search. This represents the information
/// for a certain depth.
/// - `not_pruned` is the number of nodes that were actually searched at a
///   certain depth.
///   - `expanded` is the number of nodes (of the `not_pruned` nodes) that were
///      actually expanded (rather than being resolved by a table lookup).
///  - `pruned` is the number of nodes that were never searched for a given
///    depth, because the were pruned.
#[derive(Clone, Copy)]
struct BranchInfo {
    pub not_pruned: u64,
    pub expanded: u64,
    pub pruned: u64,
}
impl BranchInfo {
    fn new() -> Self {
        BranchInfo {
            not_pruned: 0,
            expanded: 0,
            pruned: 0,
        }
    }
}

impl AlphaBeta {

    pub fn new(static_evaluator: impl StaticEvaluator + 'static, lookahead: u8) -> Self {
        assert!(lookahead > 0, "lookahead must be positive");
        AlphaBeta {
            static_evaluator: Box::new(static_evaluator),
            lookahead,
            position_table: PositionTable::new(),
            rounding_errors: 0,
            branch_info: vec![BranchInfo::new(); lookahead as usize + 1],
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
    /// - If `depth` is 0, then the `Move` will not be contained in the result
    /// - If `depth` is `self.lookahead`, then the `Move` will be contained in
    ///     the result
    /// - Otherwise, the `Move` may or may not be contained in the result
    ///     depending on whether the evaluation came from the position table
    fn get_scored_best_move(&mut self, board: &MyBoard, bounds: Bounds, depth: u8) -> SearchResult {
        assert!(bounds.valid());
        let mut bounds = bounds;

        self.branch_info[depth as usize].not_pruned += 1;

        // Check if there is an existing entry in the position table
        if let Some(score_info) = self.position_table.get(board, depth) {
            if bounds.info_too_low(score_info) { return Low; }
            else if bounds.info_too_high(score_info) {
                assert!(bounds.max != None, "info too high with no max bound");
                return High; }
            else if let Some(score) = score_info.actual_score() {
                if depth != self.lookahead {
                    // Unfortunately we can't use the table for the root,
                    // since it doesn't contain the move required
                    return Result(score, None);
                }
            }
            // Updating the bounds here should be possible, but it's fraught,
            // since if we get an evaluation that is at a higher depth,
            // we might be updating them to be too tight which could result
            // in an incorrectly returned prune. So we don't do that.
        }

        self.branch_info[depth as usize].expanded += 1;

        if depth == 0 || !matches!(board.get_status(), Status::InProgress) {
            let evaluation = self.static_evaluator.evaluate(board);

            self.position_table.insert(
                board, depth, ScoreInfo::from_score(evaluation));

            return
                if bounds.score_too_low(evaluation) { Low }
                else if bounds.score_too_high(evaluation) { High }
                else { Result(evaluation, None) }
        }

        let is_maxing = matches!(board.get_side_to_move(), White);
        let mut best_result = None;

        let moves = board.all_moves();
        let n_moves = moves.len() as u64;

        for (i, mv) in moves.into_iter().enumerate() { // TODO: order moves (or iterative deepening)
            
            let (b_board, nb_board) = self.next_boards(board, mv, depth != 1);

            // Calculate the implied bounds on the no-bonus branch, assuming
            // a worst-case scenario for the bonus branch at both sides of the
            // bound.
            let nb_bounds = bounds
                .min_decreased_by(crate::bonus_chance())
                .expanded(crate::no_bonus_chance());
            
            let nb_result = self.get_scored_best_move(&nb_board, nb_bounds, depth - 1);
            
            // Determine a probability weighted score for this move, or a prune
            let result = if let Result(nb_score, _) = nb_result {
                let b_bounds = bounds
                    .both_decreased_by(nb_score * crate::no_bonus_chance())
                    .expanded(crate::bonus_chance());
                let b_result = self.get_scored_best_move(&b_board, b_bounds, depth - 1);
                if let Result(b_score, _) = b_result {
                    let score =
                        b_score * crate::bonus_chance()
                        + nb_score * crate::no_bonus_chance();
                    if !bounds.contains(score) {
                        self.rounding_errors += 1;
                        if Some(score) == bounds.min { Low }
                        else if Some(score) == bounds.max { High }
                        else { panic!("score is distinctly out of bounds"); }
                    } else {
                        Result(score, None)
                    }
                } else { b_result }
            } else { nb_result };

            // Set `score` to be the actual score, unless it was a prune, in
            // which case we either continue or return, depending on the
            // direction of the prune
            let Result(score, _) = result else {
                if is_maxing == matches!(result, Low) { continue; }
                else {
                    let res = if is_maxing { High } else { Low };
                    self.update_table_for_result(board, depth, bounds, &res);
                    let num_pruned = n_moves - (i as u64) - 1;
                    self.branch_info[depth as usize - 1].pruned += num_pruned;
                    return res;
                }
            };

            assert!(bounds.contains(score), "bounds should contain score \
                because of the bounds on nb_score and b_score");

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

        let res = if let Some((score, mv)) = best_result {
            Result(score, Some(mv))
        } else {
            if is_maxing { Low } else { High }
        };
        self.update_table_for_result(board, depth, bounds, &res);
        res
    }

    fn update_table_for_result(&mut self,
        board: &MyBoard, depth: u8, bounds: Bounds, result: &SearchResult
    ) {
        // We don't do anything fancy by merging the new result with the old
        // since bugs can arise from that due to the fact that the new result
        // might be referencing table entries which the old result couldn't.
        // This could lead to incompatible ranges.
        let new = match result {
            Result(score, _) => ScoreInfo::from_score(*score),
            Low => ScoreInfo::from_max_score(bounds.min.expect("\
                shouldn't return Low if there is no minimum bound\
            ")),
            High => ScoreInfo::from_min_score(bounds.max.expect("\
                shouldn't return High if there is no maximum bound\
            ")),
        };
        self.position_table.insert(board, depth, new);
    }

    fn get_line(&mut self, board: &MyBoard) -> js_sys::Array {
        let line = js_sys::Array::new();

        if let Result(score, None) =
            self.get_scored_best_move(board, Bounds::widest(), self.lookahead)
        {
            line.push(&format!("score: {}", score).into());
        } else {

            let Result(score, Some(mv)) =
                self.get_scored_best_move(board, Bounds::widest(), self.lookahead)
                else { panic!(); };
            
            line.push(&format!("score: {}", score).into());
            
            self.lookahead -= 1; // we adjust this as we recurse

            line.push(&format!("side to move: {:?}",
                board.get_side_to_move()).into());
            
            line.push(&format!("best move: {} to {}", mv.get_source(), mv.get_dest()).into());

            let (b_board, nb_board) = self.next_boards(board, mv, true);
            line.push(&self.get_line(&nb_board));
            line.push(&self.get_line(&b_board));

            self.lookahead += 1;
        }

        line
    }

    fn prune_statistics(&mut self) -> String {
        let mut s = String::new();
        
        s.push_str(&format!("Pruning statistics:\n"));

        for depth in (0..self.lookahead as usize + 1).rev() {
        
            let d = self.lookahead as usize - depth;

            let np = self.branch_info[depth].not_pruned;
            let p = self.branch_info[depth].pruned;
            let e = self.branch_info[depth].expanded;
            let t = np + p;
            let l = np - e;

            if depth == self.lookahead as usize {
                s.push_str(&format!("\tDepth {} (root) had {} nodes:\n",
                    d, t));
            } else {
                s.push_str(&format!("\tDepth {} had {} nodes (avg. branching factor of {}):\n",
                    d, t, t / self.branch_info[depth + 1].expanded));
            }

            s.push_str(&format!("\t\t{} ({}%) were expanded\n",
                e, (e*100) / t));
            s.push_str(&format!("\t\t{} ({}%) were resolved with a table lookup\n",
                l, (l * 100) / t));
            s.push_str(&format!("\t\t{} ({}%) were pruned\n",
                p, (p * 100) / t));
        }

        self.branch_info = vec![BranchInfo::new(); self.lookahead as usize + 1];
        
        s
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

    fn get_move(&mut self, board: &MyBoard) -> ChessMove {
        web_sys::console::time_with_label("calculating best move");
        let mv = match
            self.get_scored_best_move(board, Bounds::widest(), self.lookahead)
        {
            Result(_, mv) => mv.expect("move should be returned at top level"),
            _ => panic!("pruning should not happen with the widest bounds"),
        };
        web_sys::console::time_end_with_label("calculating best move");
        self.log_info();
        web_sys::console::time_with_label("generating reasoning");
        web_sys::console::log_1(&self.get_line(board));
        web_sys::console::time_end_with_label("generating reasoning");
        mv
    }

    fn log_info(&mut self) {
        web_sys::console::log_1(&self.position_table.info().into());
        self.position_table.reset_debug_info();
        web_sys::console::log_1(&format!("detected {} rounding errors", self.rounding_errors).into());
        self.rounding_errors = 0;
        web_sys::console::log_1(&self.prune_statistics().into());
    }

}