use super::*;

use acme_kit::text;

/// Where a adaptive rollback estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRollbackEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveRollbackEstimateStatus {
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

impl std::fmt::Display for AdaptiveRollbackEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive rollback estimate as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackEstimate {
    pub code: String,
    pub label: String,
    pub name: String,
    pub capacity: i64,
    pub budget: f64,
    pub cost: f64,
    pub locale: String,
    pub margin: f64,
    pub status: AdaptiveRollbackEstimateStatus,
}

impl AdaptiveRollbackEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRollbackEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRollbackEstimateStatus) -> bool {
        match self.status {
            AdaptiveRollbackEstimateStatus::Idle => {
                match next {
                    AdaptiveRollbackEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRollbackEstimateStatus::Warming => {
                return next == AdaptiveRollbackEstimateStatus::Ready;
            }
            AdaptiveRollbackEstimateStatus::Ready => {
                return next == AdaptiveRollbackEstimateStatus::Draining;
            }
            AdaptiveRollbackEstimateStatus::Draining => {
                match next {
                    AdaptiveRollbackEstimateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AdaptiveRollbackEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rollback estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive rollback estimate", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.cost + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRollbackEstimate {
        AdaptiveRollbackEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            budget: self.budget,
            cost: self.cost,
            locale: self.locale.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a adaptive rollback estimate in status idle.
pub fn new_adaptive_rollback_estimate(code: &str, label: &str) -> AdaptiveRollbackEstimate {
    AdaptiveRollbackEstimate {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        capacity: 0,
        budget: 0.0,
        cost: 0.0,
        locale: "".to_string(),
        margin: 0.0,
        status: AdaptiveRollbackEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_rollback_estimate("code-593", "label-285");
        assert_eq!(item.status, AdaptiveRollbackEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_adaptive_rollback_estimate("code-593", "label-285");
        if item.move_to(AdaptiveRollbackEstimateStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRollbackEstimateStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_rollback_estimate("code-593", "label-285");
        item.status = AdaptiveRollbackEstimateStatus::Stopped;
        assert!(
            item.move_to(AdaptiveRollbackEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
