use super::*;

use acme_kit::text;

/// Where a pending rollout review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRolloutReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingRolloutReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingRolloutReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending rollout review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutReview {
    pub code: String,
    pub label: String,
    pub title: String,
    pub budget: f64,
    pub segment: String,
    pub priority: i64,
    pub status: PendingRolloutReviewStatus,
}

impl PendingRolloutReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRolloutReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRolloutReviewStatus) -> bool {
        match self.status {
            PendingRolloutReviewStatus::Proposed => {
                match next {
                    PendingRolloutReviewStatus::Approved | PendingRolloutReviewStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRolloutReviewStatus::Approved => {
                return next == PendingRolloutReviewStatus::Rejected || next == PendingRolloutReviewStatus::Withdrawn;
            }
            PendingRolloutReviewStatus::Rejected => {
                return next == PendingRolloutReviewStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRolloutReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending rollout review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.priority as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRolloutReview {
        PendingRolloutReview {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            budget: self.budget,
            segment: self.segment.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a pending rollout review in status proposed.
pub fn new_pending_rollout_review(code: &str, label: &str) -> PendingRolloutReview {
    PendingRolloutReview {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        budget: 0.0,
        segment: "".to_string(),
        priority: 0,
        status: PendingRolloutReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_rollout_review("code-501", "label-753");
        assert_eq!(item.status, PendingRolloutReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_pending_rollout_review("code-501", "label-753");
        if item.move_to(PendingRolloutReviewStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRolloutReviewStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_rollout_review("code-501", "label-753");
        item.status = PendingRolloutReviewStatus::Withdrawn;
        assert!(
            item.move_to(PendingRolloutReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
