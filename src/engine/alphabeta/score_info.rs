use crate::{Score, ONE, ZERO};

/// Stores a pair of bounds for the score of a given position. Unlike `Bounds`,
/// the bounds are inclusive on both sides, so `ZERO` and `ONE` can be used for
/// the min and max bounds.
///
/// This is used in the position table to store the results of the search.
#[derive(Clone, Copy, Debug)]
pub struct ScoreInfo {
    pub min: Score,
    pub max: Score,
}
impl ScoreInfo {
    pub fn actual_score(self) -> Option<Score> {
        if self.min == self.max {
            Some(self.min)
        } else {
            None
        }
    }
    pub fn from_score(score: Score) -> Self {
        ScoreInfo {
            min: score,
            max: score,
        }
    }
    pub fn from_min_score(min: Score) -> Self { ScoreInfo { min, max: ONE } }
    pub fn from_max_score(max: Score) -> Self { ScoreInfo { min: ZERO, max } }
}
