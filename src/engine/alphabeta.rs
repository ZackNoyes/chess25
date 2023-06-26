
mod score_info;
use score_info::ScoreInfo;

mod bounds;
use bounds::Bounds;

mod search_result;
use search_result::SearchResult::{self, *};

mod branch_info;
use branch_info::BranchInfo;

#[cfg(test)] mod tests;

use chess::{Color::*, ChessMove};

use crate::logger::Logger;
use crate::{Score, ONE};
use crate::my_board::{MyBoard, Status};

use super::Engine;
use super::evaluator::StaticEvaluator;
use super::position_table::PositionTable;

pub struct AlphaBeta {
    static_evaluator: Box<dyn StaticEvaluator>,
    lookahead: u8,
    is_pessimistic: bool,
    position_table: PositionTable<ScoreInfo>,
    logger: Logger,
    // Debug info
    rounding_errors: u32,
    branch_info: BranchInfo,
    iter_deep_failures: u32,
    iter_deep_lookups: u32,
}

impl AlphaBeta {

    pub fn new(static_evaluator: impl StaticEvaluator + 'static,
        lookahead: u8, is_pessimistic: bool, log_level: u8
    ) -> Self {
        assert!(lookahead > 0, "lookahead must be positive");
        let logger = Logger::new(log_level);
        AlphaBeta {
            static_evaluator: Box::new(static_evaluator),
            lookahead,
            is_pessimistic,
            position_table: PositionTable::new(&logger),
            logger,
            rounding_errors: 0,
            branch_info: BranchInfo::new(lookahead),
            iter_deep_failures: 0,
            iter_deep_lookups: 0,
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
    /// 
    /// TODO: There are many more optimisations that could be done here. Areas
    /// to look at would be why the iterdeep version is slower than the normal
    /// version in ordinary cases. Perhaps a heuristic improvement would help
    /// the ordering.
    fn get_scored_best_move(&mut self, board: &MyBoard, bounds: Bounds, depth: u8) -> SearchResult {
        assert!(bounds.valid());
        assert!(depth <= self.lookahead);
        let mut bounds = bounds;

        self.branch_info[depth as usize].not_pruned += 1;

        // Check if there is an existing entry in the position table
        if let Some(score_info) = self.position_table.get(board, depth) {
            if bounds.info_too_low(score_info) { return Low; }
            else if bounds.info_too_high(score_info) { return High; }
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

        let mut moves = board.all_moves();
        let n_moves = moves.len() as u64;

        if depth > 1 {

            // sort_by_cached_key was faster than sort_unstable_by_key
            // after a few tests, so we use that
            moves.sort_by_cached_key(|mv| {

                self.iter_deep_lookups += 1;

                let (_, nb_board) = self.next_boards(board, *mv, false);

                let mut key = None;

                if let Some(info) = self.position_table.get_lenient(&nb_board) {
                    if let Some(score) = info.actual_score() {
                        key = Some(score);
                    }
                }
                
                let key = key.unwrap_or_else(|| {
                    self.iter_deep_failures += 1;
                    let eval = self.static_evaluator.evaluate(&nb_board);
                    self.position_table.insert_both_colors(&nb_board, 0, ScoreInfo::from_score(eval));
                    eval
                });

                if is_maxing { ONE - key } else { key }
            });

        }

        for (i, mv) in moves.into_iter().enumerate() {
            
            let (b_board, nb_board) = self.next_boards(board, mv, depth > 1);

            // Define the bonus and non-bonus chances in an adjusted way.
            // This has the effect of making the AI more defensive.
            // This makes it more fun to play against, and also probably more
            // consistent against weaker opponents.
            // TODO: Replace this with a better heuristic
            let mut b_chance = crate::bonus_chance();
            let mut nb_chance = crate::no_bonus_chance();

            if self.is_pessimistic {
                let adjustment = Score::from_num(
                    ((b_board.get_black_pieces() | b_board.get_white_pieces())
                    .count()) as f64 / 200.0
                );
                if is_maxing {
                    b_chance += adjustment;
                    nb_chance -= adjustment;
                } else {
                    b_chance -= adjustment;
                    nb_chance += adjustment;
                }
            }

            // Calculate the implied bounds on the no-bonus branch, assuming
            // a worst-case scenario for the bonus branch at both sides of the
            // bound.
            let nb_bounds = bounds
                .min_decreased_by(b_chance)
                .expanded(nb_chance);
            
            let nb_result = self.get_scored_best_move(&nb_board, nb_bounds, depth - 1);
            
            // Determine a probability weighted score for this move, or a prune
            let result = if let Result(nb_score, _) = nb_result {
                let b_bounds = bounds
                    .both_decreased_by(nb_score * nb_chance)
                    .expanded(b_chance);
                let b_result = self.get_scored_best_move(&b_board, b_bounds, depth - 1);
                if let Result(b_score, _) = b_result {
                    let score =
                        b_score * b_chance
                        + nb_score * nb_chance;
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
        } else if is_maxing { Low } else { High };

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

}

impl Engine for AlphaBeta {

    fn default(static_evaluator: impl StaticEvaluator + 'static) -> Self {
        AlphaBeta::new(static_evaluator, 4, false, 10)
    }

    fn evaluate(&mut self, board: &MyBoard) -> Score {
        match self.get_scored_best_move(board, Bounds::widest(), self.lookahead) {
            Result(score, _) => score,
            _ => panic!("pruning should not happen with the widest bounds"),
        }
    }

    fn get_move(&mut self, board: &MyBoard) -> ChessMove {
        self.logger.time_start(2, "move calculation");

        for depth in 1..self.lookahead {

            self.logger.time_start(5, &format!("depth {}", depth));

            self.iter_deep_lookups = 0;
            self.iter_deep_failures = 0;

            let s = match
                self.get_scored_best_move(board, Bounds::widest(), depth)
            {
                Result(s, _) => s,
                _ => panic!("pruning should not happen with the widest bounds"),
            };
            
            self.logger.log(5, &format!("\
                depth {}: score {}\n\t{}/{} ({}%) lookup failures",
                depth, s, self.iter_deep_failures, self.iter_deep_lookups,
                (self.iter_deep_failures * 100) / (self.iter_deep_lookups + 1)
            ));

            self.logger.time_end(5, &format!("depth {}", depth));

        }

        self.position_table.reset_debug_info();
        self.rounding_errors = 0;
        self.branch_info.reset_statistics();

        
        self.logger.time_start(5, &format!("depth {} (final)", self.lookahead));
        
        let (s, mv) = match
            self.get_scored_best_move(board, Bounds::widest(), self.lookahead)
        {
            Result(s, mv) => (s, mv.expect("move should be returned at top level")),
            _ => panic!("pruning should not happen with the widest bounds"),
        };
        
        
        self.logger.time_end(5, &format!("depth {} (final)", self.lookahead));

        self.logger.log(2, &format!("{:?} to move evaluation: {}",
            board.get_side_to_move(), s));

        self.logger.time_end(2, "move calculation");

        self.logger.time_start(8, "reasoning generation");
        self.logger.clone().log_lazy_arr(8, || { self.get_line(board) });
        self.logger.time_end(8, "reasoning generation");
        
        mv
    }

    fn log_info(&self) {
        self.logger.log_lazy(5, || { self.position_table.info() });
        self.logger.log(5, &format!("detected {} rounding errors", self.rounding_errors));
        self.logger.log_lazy(5, || { self.branch_info.statistics() });
    }

    fn get_logger(&self) -> &Logger {
        &self.logger
    }

}