use super::*;

use acme_kit::text;

/// Where a strict release review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictReleaseReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictReleaseReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for StrictReleaseReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict release review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleaseReview {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub visible: bool,
    pub fingerprint: String,
    pub ceiling: f64,
    pub rate: f64,
    pub namespace: String,
    pub revision: i64,
    pub status: StrictReleaseReviewStatus,
}

impl StrictReleaseReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictReleaseReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictReleaseReviewStatus) -> bool {
        match self.status {
            StrictReleaseReviewStatus::Planned => {
                return next == StrictReleaseReviewStatus::Scheduled || next == StrictReleaseReviewStatus::InProgress;
            }
            StrictReleaseReviewStatus::Scheduled => {
                return next == StrictReleaseReviewStatus::InProgress;
            }
            StrictReleaseReviewStatus::InProgress => {
                match next {
                    StrictReleaseReviewStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictReleaseReviewStatus::Completed => {
                return next == StrictReleaseReviewStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictReleaseReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict release review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ceiling + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictReleaseReview {
        StrictReleaseReview {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            visible: self.visible,
            fingerprint: self.fingerprint.clone(),
            ceiling: self.ceiling,
            rate: self.rate,
            namespace: self.namespace.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a strict release review in status planned.
pub fn new_strict_release_review(code: &str, label: &str) -> StrictReleaseReview {
    StrictReleaseReview {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        visible: false,
        fingerprint: "".to_string(),
        ceiling: 0.0,
        rate: 0.0,
        namespace: "".to_string(),
        revision: 0,
        status: StrictReleaseReviewStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_strict_release_review("code-606", "label-198");
        assert_eq!(item.status, StrictReleaseReviewStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_strict_release_review("code-606", "label-198");
        if item.move_to(StrictReleaseReviewStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictReleaseReviewStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_strict_release_review("code-606", "label-198");
        item.status = StrictReleaseReviewStatus::Cancelled;
        assert!(
            item.move_to(StrictReleaseReviewStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
