use super::*;

use acme_kit::text;

/// Where a lenient zone review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientZoneReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientZoneReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientZoneReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient zone review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneReview {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub window_days: i64,
    pub priority: i64,
    pub threshold: f64,
    pub external_ref: String,
    pub status: LenientZoneReviewStatus,
}

impl LenientZoneReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientZoneReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientZoneReviewStatus) -> bool {
        match self.status {
            LenientZoneReviewStatus::Proposed => {
                match next {
                    LenientZoneReviewStatus::Rejected | LenientZoneReviewStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientZoneReviewStatus::Approved => {
                return next == LenientZoneReviewStatus::Rejected;
            }
            LenientZoneReviewStatus::Rejected => {
                return next == LenientZoneReviewStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientZoneReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient zone review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.priority as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientZoneReview {
        LenientZoneReview {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            window_days: self.window_days,
            priority: self.priority,
            threshold: self.threshold,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient zone review in status proposed.
pub fn new_lenient_zone_review(code: &str, label: &str) -> LenientZoneReview {
    LenientZoneReview {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        window_days: 0,
        priority: 0,
        threshold: 0.0,
        external_ref: "".to_string(),
        status: LenientZoneReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_zone_review("code-438", "label-915");
        assert_eq!(item.status, LenientZoneReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_lenient_zone_review("code-438", "label-915");
        if item.move_to(LenientZoneReviewStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientZoneReviewStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_zone_review("code-438", "label-915");
        item.status = LenientZoneReviewStatus::Withdrawn;
        assert!(
            item.move_to(LenientZoneReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
