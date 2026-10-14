use super::*;

use acme_kit::text;

/// Where a probe review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ProbeReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ProbeReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe review as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeReview {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub sequence: i64,
    pub category: String,
    pub status: ProbeReviewStatus,
}

impl ProbeReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeReviewStatus) -> bool {
        match self.status {
            ProbeReviewStatus::Proposed => {
                match next {
                    ProbeReviewStatus::Rejected | ProbeReviewStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeReviewStatus::Approved => {
                match next {
                    ProbeReviewStatus::Rejected | ProbeReviewStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeReviewStatus::Rejected => {
                return next == ProbeReviewStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> ProbeReview {
        ProbeReview {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            sequence: self.sequence,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a probe review in status proposed.
pub fn new_probe_review(code: &str, label: &str) -> ProbeReview {
    ProbeReview {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        sequence: 0,
        category: "".to_string(),
        status: ProbeReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_probe_review("code-769", "label-585");
        assert_eq!(item.status, ProbeReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_probe_review("code-769", "label-585");
        if item.move_to(ProbeReviewStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeReviewStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_probe_review("code-769", "label-585");
        item.status = ProbeReviewStatus::Withdrawn;
        assert!(
            item.move_to(ProbeReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
