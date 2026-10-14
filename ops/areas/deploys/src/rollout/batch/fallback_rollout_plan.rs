use super::*;

use acme_kit::text;

/// Where a fallback rollout plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRolloutPlanStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackRolloutPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for FallbackRolloutPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback rollout plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutPlan {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub external_ref: String,
    pub rate: f64,
    pub segment: String,
    pub status: FallbackRolloutPlanStatus,
}

impl FallbackRolloutPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRolloutPlanStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRolloutPlanStatus) -> bool {
        match self.status {
            FallbackRolloutPlanStatus::Open => {
                return next == FallbackRolloutPlanStatus::Resolved || next == FallbackRolloutPlanStatus::Archived;
            }
            FallbackRolloutPlanStatus::InReview => {
                match next {
                    FallbackRolloutPlanStatus::Resolved | FallbackRolloutPlanStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRolloutPlanStatus::Resolved => {
                return next == FallbackRolloutPlanStatus::Reopened || next == FallbackRolloutPlanStatus::Archived;
            }
            FallbackRolloutPlanStatus::Reopened => {
                match next {
                    FallbackRolloutPlanStatus::Archived => {
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

    pub fn move_to(&mut self, next: FallbackRolloutPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback rollout plan {} cannot move from {} to {}",
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
        (self.rate * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRolloutPlan {
        FallbackRolloutPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback rollout plan in status open.
pub fn new_fallback_rollout_plan(code: &str, name: &str) -> FallbackRolloutPlan {
    FallbackRolloutPlan {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        external_ref: "".to_string(),
        rate: 0.0,
        segment: "".to_string(),
        status: FallbackRolloutPlanStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_rollout_plan("code-171", "name-139");
        assert_eq!(item.status, FallbackRolloutPlanStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_fallback_rollout_plan("code-171", "name-139");
        if item.move_to(FallbackRolloutPlanStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRolloutPlanStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_rollout_plan("code-171", "name-139");
        item.status = FallbackRolloutPlanStatus::Archived;
        assert!(
            item.move_to(FallbackRolloutPlanStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
