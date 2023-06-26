use crate::{Score, ONE, ZERO, DELTA};
use super::score_info::ScoreInfo;

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
    pub fn widest() -> Bounds {
        Bounds { min: None, max: None }
    }
    pub fn score_too_low(self, score: Score) -> bool {
        if let Some(min) = self.min { score <= min } else { false }
    }
    pub fn score_too_high(self, score: Score) -> bool {
        if let Some(max) = self.max { max <= score } else { false }
    }
    pub fn contains(self, score: Score) -> bool {
        !self.score_too_low(score) && !self.score_too_high(score)
    }
    pub fn min_decreased_by(self, amount: Score) -> Self {
        Bounds {
            min: if let Some(min) = self.min {
                min.checked_sub(amount)
            } else { None },
            max: self.max,
        }
    }
    /// Divides the bounds by the given amount. Slightly confusingly, I called
    /// this `expanded` because it is called with arguments less than 1.
    pub fn expanded(self, amount: Score) -> Self {
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
                max.checked_div(amount).filter(|&new_max| new_max <= ONE)
            } else { None },
        }
    }
    pub fn both_decreased_by(self, amount: Score) -> Self {
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
    pub fn valid(self) -> bool {
        if let Some(max) = self.max {
            max <= ONE &&
            if let Some(min) = self.min {
                min < max
            } else { true }
        } else { true }
    }
    pub fn update_min(&mut self, score: Score) {
        if let Some(min) = self.min {
            if score > min { self.min = Some(score); }
        } else { self.min = Some(score); }
    }
    pub fn update_max(&mut self, score: Score) {
        if let Some(max) = self.max {
            if score < max { self.max = Some(score); }
        } else { self.max = Some(score); }
    }
    pub fn info_too_low(self, score_info: ScoreInfo) -> bool {
        if let Some(min) = self.min {
            if score_info.max <= min { return true; }
        }
        false
    }
    pub fn info_too_high(self, score_info: ScoreInfo) -> bool {
        if let Some(max) = self.max {
            if score_info.min >= max { return true; }
        }
        false
    }
}