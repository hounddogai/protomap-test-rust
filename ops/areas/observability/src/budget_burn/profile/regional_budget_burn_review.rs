use super::*;

use acme_kit::text;

/// Where a regional budget burn review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalBudgetBurnReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalBudgetBurnReviewStatus {
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

impl std::fmt::Display for RegionalBudgetBurnReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional budget burn review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnReview {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub rotation_days: i64,
    pub cost: f64,
    pub capacity: i64,
    pub scope: String,
    pub minimum: f64,
    pub threshold: f64,
    pub grace_days: i64,
    pub fingerprint: String,
    pub status: RegionalBudgetBurnReviewStatus,
}

impl RegionalBudgetBurnReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalBudgetBurnReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalBudgetBurnReviewStatus) -> bool {
        match self.status {
            RegionalBudgetBurnReviewStatus::Idle => {
                match next {
                    RegionalBudgetBurnReviewStatus::Warming | RegionalBudgetBurnReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalBudgetBurnReviewStatus::Warming => {
                match next {
                    RegionalBudgetBurnReviewStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalBudgetBurnReviewStatus::Ready => {
                return next == RegionalBudgetBurnReviewStatus::Draining;
            }
            RegionalBudgetBurnReviewStatus::Draining => {
                return next == RegionalBudgetBurnReviewStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalBudgetBurnReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional budget burn review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.floor_value + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalBudgetBurnReview {
        RegionalBudgetBurnReview {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            rotation_days: self.rotation_days,
            cost: self.cost,
            capacity: self.capacity,
            scope: self.scope.clone(),
            minimum: self.minimum,
            threshold: self.threshold,
            grace_days: self.grace_days,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a regional budget burn review in status idle.
pub fn new_regional_budget_burn_review(code: &str, name: &str) -> RegionalBudgetBurnReview {
    RegionalBudgetBurnReview {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        rotation_days: 0,
        cost: 0.0,
        capacity: 0,
        scope: "".to_string(),
        minimum: 0.0,
        threshold: 0.0,
        grace_days: 0,
        fingerprint: "".to_string(),
        status: RegionalBudgetBurnReviewStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_budget_burn_review("code-546", "name-402");
        assert_eq!(item.status, RegionalBudgetBurnReviewStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_regional_budget_burn_review("code-546", "name-402");
        if item.move_to(RegionalBudgetBurnReviewStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalBudgetBurnReviewStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_budget_burn_review("code-546", "name-402");
        item.status = RegionalBudgetBurnReviewStatus::Stopped;
        assert!(
            item.move_to(RegionalBudgetBurnReviewStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
