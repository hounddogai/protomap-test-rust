use super::*;

use acme_kit::text;

/// Where a scheduled verifier review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledVerifierReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledVerifierReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledVerifierReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled verifier review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierReview {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub ratio: f64,
    pub updated_at: i64,
    pub status: ScheduledVerifierReviewStatus,
}

impl ScheduledVerifierReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledVerifierReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledVerifierReviewStatus) -> bool {
        match self.status {
            ScheduledVerifierReviewStatus::Proposed => {
                match next {
                    ScheduledVerifierReviewStatus::Approved | ScheduledVerifierReviewStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledVerifierReviewStatus::Approved => {
                return next == ScheduledVerifierReviewStatus::Rejected
                    || next == ScheduledVerifierReviewStatus::Withdrawn;
            }
            ScheduledVerifierReviewStatus::Rejected => {
                match next {
                    ScheduledVerifierReviewStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledVerifierReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled verifier review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled verifier review", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledVerifierReview {
        ScheduledVerifierReview {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            ratio: self.ratio,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled verifier review in status proposed.
pub fn new_scheduled_verifier_review(code: &str, label: &str) -> ScheduledVerifierReview {
    ScheduledVerifierReview {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        ratio: 0.0,
        updated_at: 0,
        status: ScheduledVerifierReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_verifier_review("code-60", "label-741");
        assert_eq!(item.status, ScheduledVerifierReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_scheduled_verifier_review("code-60", "label-741");
        if item.move_to(ScheduledVerifierReviewStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledVerifierReviewStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_verifier_review("code-60", "label-741");
        item.status = ScheduledVerifierReviewStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledVerifierReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
