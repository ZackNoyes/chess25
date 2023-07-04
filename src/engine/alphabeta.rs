mod score_info;
use score_info::ScoreInfo;

mod bounds;
use bounds::Bounds;

mod search_result;
use search_result::SearchResult::{self, *};

mod branch_info;
use branch_info::BranchInfo;

#[cfg(test)] mod tests;

use chess::{ChessMove, Color::*};
use either::Either::{Left, Right};

use super::{evaluator::StaticEvaluator, position_table::PositionTable, Engine};
use crate::{deadline::Deadline, logger::Logger, my_board::MyBoard, Score, ONE};

pub struct AlphaBeta {
    static_evaluator: Box<dyn StaticEvaluator>,
    max_lookahead: u8,
    max_time: u64,
    is_pessimistic: bool,
    is_focussed: bool,
    position_table: PositionTable<ScoreInfo>,
    logger: Logger,
    // Debug info
    branch_info: BranchInfo,
    iter_deep_failures: u32,
    iter_deep_lookups: u32,
}

impl AlphaBeta {
    /// Using a larger log level may have performance costs
    pub fn new(
        static_evaluator: impl StaticEvaluator + 'static, max_lookahead: u8, is_pessimistic: bool,
        is_focussed: bool, log_level: u8, max_time: u64,
    ) -> Self {
        assert!(max_lookahead > 0, "lookahead must be positive");
        assert!(
            !is_focussed || max_lookahead > 1,
            "lookahead must be greater than 1 if focussed"
        );
        let logger = Logger::new(log_level);
        AlphaBeta {
            static_evaluator: Box::new(static_evaluator),
            max_lookahead,
            max_time,
            is_pessimistic,
            is_focussed,
            position_table: PositionTable::new(&logger),
            logger,
            branch_info: BranchInfo::new(max_lookahead),
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
    /// - If `get_move` is `true`, then the `Move` is guaranteed to be in the
    ///   result.
    /// - Otherwise, the `Move` may or may not be contained in the result
    ///   depending on whether the evaluation came from the position table
    fn get_scored_best_move(
        &mut self, board: &MyBoard, bounds: Bounds, depth: u8, get_move: bool, deadline: Deadline,
    ) -> SearchResult {
        assert!(bounds.valid());

        if deadline.expired() {
            return Timeout;
        }

        let mut bounds = bounds;

        let finish_depth = if self.is_focussed { 1 } else { 0 };

        self.branch_info[depth as usize].not_pruned += 1;

        // Check if there is an existing entry in the position table
        if let Some(score_info) = self.position_table.get(board, depth) {
            if bounds.info_too_low(score_info) {
                return Low;
            } else if bounds.info_too_high(score_info) {
                return High;
            } else if let Some(score) = score_info.actual_score() {
                if !get_move {
                    return Result(score, None);
                }
            }
            // Updating the bounds here should be possible, but it's fraught,
            // since if we get an evaluation that is at a higher depth,
            // we might be updating them to be too tight which could result
            // in an incorrectly returned prune. So we don't do that.
        }

        self.branch_info[depth as usize].expanded += 1;

        if depth <= finish_depth || !board.get_status().is_in_progress() {
            let evaluation = self.static_evaluator.evaluate(board);

            // TODO: Take advantage of the fact that a lot of the computation when just the
            //   side to move changes is redundant (see below)
            self.position_table
                .insert(board, depth, ScoreInfo::from_score(evaluation));

            return if bounds.score_too_low(evaluation) {
                Low
            } else if bounds.score_too_high(evaluation) {
                High
            } else {
                assert!(!get_move, "depth was too small to return a move");
                Result(evaluation, None)
            };
        }

        let is_maxing = board.get_side_to_move() == White;
        let mut best_result = None;

        let moves = if depth > finish_depth + 1 {
            let mut moves: Vec<_> = board.all_moves().collect();
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
                    // TODO: Take advantage of the fact that a lot of the computation when just the
                    //   side to move changes is redundant (see above)
                    self.position_table.insert(
                        &nb_board,
                        finish_depth,
                        ScoreInfo::from_score(eval),
                    );
                    eval
                });

                if is_maxing {
                    ONE - key
                } else {
                    key
                }
            });

            Left(moves.into_iter())
        } else {
            Right(board.all_moves())
        };

        for mv in moves {
            let (b_board, nb_board) = self.next_boards(board, mv, depth > finish_depth + 1);

            // Define the bonus and non-bonus chances in an adjusted way.
            // This has the effect of making the AI more defensive.
            // This makes it more fun to play against, and also probably more
            // consistent against weaker opponents.
            let mut b_chance = crate::bonus_chance();
            let mut nb_chance = crate::no_bonus_chance();

            if self.is_pessimistic {
                let adjustment = Score::from_num(
                    ((b_board.get_black_pieces() | b_board.get_white_pieces()).count()) as f64
                        / 200.0,
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
            let nb_bounds = bounds.min_decreased_by(b_chance).expanded(nb_chance);

            let nb_result =
                self.get_scored_best_move(&nb_board, nb_bounds, depth - 1, false, deadline);

            // Determine a probability weighted score for this move, or a prune
            let result = if let Result(nb_score, _) = nb_result {
                let b_bounds = bounds
                    .both_decreased_by(nb_score * nb_chance)
                    .expanded(b_chance);
                let b_result = self.get_scored_best_move(
                    &b_board,
                    b_bounds,
                    depth - if self.is_focussed { 2 } else { 1 },
                    false,
                    deadline,
                );
                if let Result(b_score, _) = b_result {
                    let score = b_score * b_chance + nb_score * nb_chance;
                    if !bounds.contains(score) {
                        if Some(score) == bounds.min {
                            Low
                        } else if Some(score) == bounds.max {
                            High
                        } else {
                            panic!("score is distinctly out of bounds");
                        }
                    } else {
                        Result(score, None)
                    }
                } else {
                    b_result
                }
            } else {
                nb_result
            };

            // Set `score` to be the actual score, unless it was a prune, in
            // which case we either continue or return, depending on the
            // direction of the prune
            let Result(score, _) = result else {
                if result == Timeout {
                    return Timeout;
                }
                if is_maxing == (result == Low) { continue; }
                else {
                    let res = if is_maxing { High } else { Low };
                    self.update_table_for_result(board, depth, bounds, &res);
                    self.branch_info[depth as usize].prunes += 1;
                    return res;
                }
            };

            assert!(
                bounds.contains(score),
                "bounds should contain score \
                because of the bounds on nb_score and b_score"
            );

            // Update the bounds with this new result
            if is_maxing {
                bounds.update_min(score);
            } else {
                bounds.update_max(score);
            }

            // Update the best result found so far
            best_result = match best_result {
                None => Some((score, mv)),
                Some((best_score, _))
                    if (is_maxing && score > best_score) || (!is_maxing && score < best_score) =>
                {
                    Some((score, mv))
                }
                _ => best_result,
            };
        }

        let res = if let Some((score, mv)) = best_result {
            Result(score, Some(mv))
        } else if is_maxing {
            Low
        } else {
            High
        };

        self.update_table_for_result(board, depth, bounds, &res);
        res
    }

    fn update_table_for_result(
        &mut self, board: &MyBoard, depth: u8, bounds: Bounds, result: &SearchResult,
    ) {
        // We don't do anything fancy by merging the new result with the old
        // since bugs can arise from that due to the fact that the new result
        // might be referencing table entries which the old result couldn't.
        // This could lead to incompatible ranges.
        let new = match result {
            Result(score, _) => ScoreInfo::from_score(*score),
            Low => ScoreInfo::from_max_score(
                bounds
                    .min
                    .expect("shouldn't return Low if there is no minimum bound"),
            ),
            High => ScoreInfo::from_min_score(
                bounds
                    .max
                    .expect("shouldn't return High if there is no maximum bound"),
            ),
            Timeout => return,
        };
        self.position_table.insert(board, depth, new);
    }
}

impl Engine for AlphaBeta {
    fn default(static_evaluator: impl StaticEvaluator + 'static) -> Self {
        AlphaBeta::new(static_evaluator, 4, false, false, 10, 10000)
    }

    fn evaluate(&mut self, _board: &MyBoard) -> Score {
        unimplemented!();
    }

    fn get_move(&mut self, board: &MyBoard) -> ChessMove {
        self.logger
            .log_lazy(5, || format!("Getting move for board:\n{}", board));

        self.logger.time_start(2, "full move calculation");
        let deadline = Deadline::from_now(self.max_time);

        let mut best_move = None;

        for depth in 2..=self.max_lookahead {
            self.iter_deep_lookups = 0;
            self.iter_deep_failures = 0;
            self.position_table.reset_debug_info();
            self.branch_info.reset_statistics();

            self.logger.time_start(4, &format!("depth {}", depth));

            let (s, mv) =
                match self.get_scored_best_move(board, Bounds::widest(), depth, true, deadline) {
                    Result(s, Some(mv)) => (s, mv),
                    Timeout => {
                        self.logger.log(4, &format!("depth {}: timeout", depth));
                        self.logger.time_end(4, &format!("depth {}", depth));
                        break;
                    }
                    _ => panic!("actual move should be returned"),
                };

            self.logger
                .log(4, &format!("depth {}: move {} with score {}", depth, mv, s));

            best_move = Some((mv, s, depth));

            self.logger.time_end(4, &format!("depth {}", depth));
            self.log_info();
        }

        self.logger.time_end(2, "full move calculation");

        let best_move = best_move.expect("could not find a move in the time/lookahead given");

        self.logger.log(
            2,
            &format!(
                "Reached depth {} and found move {} with score {}",
                best_move.2, best_move.0, best_move.1
            ),
        );

        best_move.0
    }

    fn log_info(&self) {
        self.logger.log_lazy(6, || {
            format!(
                "{} lookups, {} ({}%) failures",
                self.iter_deep_lookups,
                self.iter_deep_failures,
                (self.iter_deep_failures * 100)
                    .checked_div(self.iter_deep_lookups)
                    .unwrap_or(0),
            )
        });
        self.logger.log_lazy(6, || self.position_table.info());
        self.logger.log_lazy(6, || self.branch_info.statistics());
    }

    fn get_logger(&self) -> &Logger { &self.logger }
}
