use super::*;

use acme_kit::text;

/// Where a peering review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PeeringReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PeeringReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringReview {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub quota_units: i64,
    pub threshold: f64,
    pub batch_size: i64,
    pub port: i64,
    pub pinned: bool,
    pub checksum: String,
    pub source: String,
    pub currency: String,
    pub status: PeeringReviewStatus,
}

impl PeeringReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringReviewStatus) -> bool {
        match self.status {
            PeeringReviewStatus::Proposed => {
                return next == PeeringReviewStatus::Rejected;
            }
            PeeringReviewStatus::Approved => {
                return next == PeeringReviewStatus::Rejected || next == PeeringReviewStatus::Withdrawn;
            }
            PeeringReviewStatus::Rejected => {
                match next {
                    PeeringReviewStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PeeringReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "peering review", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quota_units as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringReview {
        PeeringReview {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            quota_units: self.quota_units,
            threshold: self.threshold,
            batch_size: self.batch_size,
            port: self.port,
            pinned: self.pinned,
            checksum: self.checksum.clone(),
            source: self.source.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a peering review in status proposed.
pub fn new_peering_review(code: &str, label: &str) -> PeeringReview {
    PeeringReview {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        quota_units: 0,
        threshold: 0.0,
        batch_size: 0,
        port: 0,
        pinned: false,
        checksum: "".to_string(),
        source: "".to_string(),
        currency: "".to_string(),
        status: PeeringReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_peering_review("code-356", "label-745");
        assert_eq!(item.status, PeeringReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_peering_review("code-356", "label-745");
        if item.move_to(PeeringReviewStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringReviewStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_peering_review("code-356", "label-745");
        item.status = PeeringReviewStatus::Withdrawn;
        assert!(
            item.move_to(PeeringReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
