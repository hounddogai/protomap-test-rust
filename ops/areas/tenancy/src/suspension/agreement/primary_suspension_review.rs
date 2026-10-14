use super::*;

use acme_kit::text;

/// Where a primary suspension review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimarySuspensionReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimarySuspensionReviewStatus {
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

impl std::fmt::Display for PrimarySuspensionReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary suspension review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionReview {
    pub code: String,
    pub label: String,
    pub region: String,
    pub title: String,
    pub external_ref: String,
    pub checksum: String,
    pub score: f64,
    pub status: PrimarySuspensionReviewStatus,
}

impl PrimarySuspensionReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimarySuspensionReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimarySuspensionReviewStatus) -> bool {
        match self.status {
            PrimarySuspensionReviewStatus::Planned => {
                return next == PrimarySuspensionReviewStatus::InProgress;
            }
            PrimarySuspensionReviewStatus::Scheduled => {
                return next == PrimarySuspensionReviewStatus::Completed
                    || next == PrimarySuspensionReviewStatus::Cancelled;
            }
            PrimarySuspensionReviewStatus::InProgress => {
                return next == PrimarySuspensionReviewStatus::Completed;
            }
            PrimarySuspensionReviewStatus::Completed => {
                return next == PrimarySuspensionReviewStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimarySuspensionReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary suspension review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary suspension review", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> PrimarySuspensionReview {
        PrimarySuspensionReview {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            checksum: self.checksum.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary suspension review in status planned.
pub fn new_primary_suspension_review(code: &str, label: &str) -> PrimarySuspensionReview {
    PrimarySuspensionReview {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        title: "".to_string(),
        external_ref: "".to_string(),
        checksum: "".to_string(),
        score: 0.0,
        status: PrimarySuspensionReviewStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_suspension_review("code-844", "label-802");
        assert_eq!(item.status, PrimarySuspensionReviewStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_primary_suspension_review("code-844", "label-802");
        if item.move_to(PrimarySuspensionReviewStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimarySuspensionReviewStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_suspension_review("code-844", "label-802");
        item.status = PrimarySuspensionReviewStatus::Cancelled;
        assert!(
            item.move_to(PrimarySuspensionReviewStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
