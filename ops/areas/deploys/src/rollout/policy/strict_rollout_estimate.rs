use super::*;

use acme_kit::text;

/// Where a strict rollout estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRolloutEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictRolloutEstimateStatus {
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

impl std::fmt::Display for StrictRolloutEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict rollout estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRolloutEstimate {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub cluster: String,
    pub discount: f64,
    pub expires_at: i64,
    pub priority: i64,
    pub status: StrictRolloutEstimateStatus,
}

impl StrictRolloutEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRolloutEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRolloutEstimateStatus) -> bool {
        match self.status {
            StrictRolloutEstimateStatus::Open => {
                match next {
                    StrictRolloutEstimateStatus::Resolved | StrictRolloutEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRolloutEstimateStatus::InReview => {
                return next == StrictRolloutEstimateStatus::Resolved || next == StrictRolloutEstimateStatus::Reopened;
            }
            StrictRolloutEstimateStatus::Resolved => {
                match next {
                    StrictRolloutEstimateStatus::Reopened | StrictRolloutEstimateStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRolloutEstimateStatus::Reopened => {
                return next == StrictRolloutEstimateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictRolloutEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict rollout estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "strict rollout estimate", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.expires_at as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictRolloutEstimate {
        StrictRolloutEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            cluster: self.cluster.clone(),
            discount: self.discount,
            expires_at: self.expires_at,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a strict rollout estimate in status open.
pub fn new_strict_rollout_estimate(code: &str, label: &str) -> StrictRolloutEstimate {
    StrictRolloutEstimate {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        cluster: "".to_string(),
        discount: 0.0,
        expires_at: 0,
        priority: 0,
        status: StrictRolloutEstimateStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_rollout_estimate("code-702", "label-332");
        assert_eq!(item.status, StrictRolloutEstimateStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_strict_rollout_estimate("code-702", "label-332");
        if item.move_to(StrictRolloutEstimateStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRolloutEstimateStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_rollout_estimate("code-702", "label-332");
        item.status = StrictRolloutEstimateStatus::Archived;
        assert!(
            item.move_to(StrictRolloutEstimateStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
