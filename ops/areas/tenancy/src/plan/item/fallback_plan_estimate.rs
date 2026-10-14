use super::*;

use acme_kit::text;

/// Where a fallback plan estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPlanEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackPlanEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FallbackPlanEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback plan estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanEstimate {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub notes: String,
    pub sequence: i64,
    pub retries: i64,
    pub status: FallbackPlanEstimateStatus,
}

impl FallbackPlanEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPlanEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPlanEstimateStatus) -> bool {
        match self.status {
            FallbackPlanEstimateStatus::New => {
                match next {
                    FallbackPlanEstimateStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPlanEstimateStatus::Queued => {
                return next == FallbackPlanEstimateStatus::Done;
            }
            FallbackPlanEstimateStatus::Running => {
                return next == FallbackPlanEstimateStatus::Done || next == FallbackPlanEstimateStatus::Failed;
            }
            FallbackPlanEstimateStatus::Done => {
                return next == FallbackPlanEstimateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPlanEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback plan estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.sequence as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPlanEstimate {
        FallbackPlanEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            notes: self.notes.clone(),
            sequence: self.sequence,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback plan estimate in status new.
pub fn new_fallback_plan_estimate(code: &str, name: &str) -> FallbackPlanEstimate {
    FallbackPlanEstimate {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        notes: "".to_string(),
        sequence: 0,
        retries: 0,
        status: FallbackPlanEstimateStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_plan_estimate("code-803", "name-450");
        assert_eq!(item.status, FallbackPlanEstimateStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_fallback_plan_estimate("code-803", "name-450");
        if item.move_to(FallbackPlanEstimateStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPlanEstimateStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_plan_estimate("code-803", "name-450");
        item.status = FallbackPlanEstimateStatus::Failed;
        assert!(
            item.move_to(FallbackPlanEstimateStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
