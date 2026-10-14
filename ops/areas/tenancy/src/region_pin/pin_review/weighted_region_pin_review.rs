use super::*;

use acme_kit::text;

/// Where a weighted region pin review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRegionPinReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedRegionPinReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for WeightedRegionPinReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted region pin review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRegionPinReview {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub amount: f64,
    pub timezone: String,
    pub principal: String,
    pub status: WeightedRegionPinReviewStatus,
}

impl WeightedRegionPinReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRegionPinReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRegionPinReviewStatus) -> bool {
        match self.status {
            WeightedRegionPinReviewStatus::Idle => {
                return next == WeightedRegionPinReviewStatus::Warming;
            }
            WeightedRegionPinReviewStatus::Warming => {
                return next == WeightedRegionPinReviewStatus::Stopped;
            }
            WeightedRegionPinReviewStatus::Ready => {
                return next == WeightedRegionPinReviewStatus::Stopped;
            }
            WeightedRegionPinReviewStatus::Draining => {
                match next {
                    WeightedRegionPinReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRegionPinReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted region pin review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> WeightedRegionPinReview {
        WeightedRegionPinReview {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            amount: self.amount,
            timezone: self.timezone.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted region pin review in status idle.
pub fn new_weighted_region_pin_review(code: &str, label: &str) -> WeightedRegionPinReview {
    WeightedRegionPinReview {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        amount: 0.0,
        timezone: "".to_string(),
        principal: "".to_string(),
        status: WeightedRegionPinReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_region_pin_review("code-834", "label-639");
        assert_eq!(item.status, WeightedRegionPinReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_region_pin_review("code-834", "label-639");
        if item.move_to(WeightedRegionPinReviewStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRegionPinReviewStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_region_pin_review("code-834", "label-639");
        item.status = WeightedRegionPinReviewStatus::Stopped;
        assert!(
            item.move_to(WeightedRegionPinReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
