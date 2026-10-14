use super::*;

use acme_kit::text;

/// Where a seasonal reservation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalReservationTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalReservationTargetStatus {
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

impl std::fmt::Display for SeasonalReservationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal reservation target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalReservationTarget {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub rotation_days: i64,
    pub name: String,
    pub approved: bool,
    pub status: SeasonalReservationTargetStatus,
}

impl SeasonalReservationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalReservationTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalReservationTargetStatus) -> bool {
        match self.status {
            SeasonalReservationTargetStatus::Open => {
                return next == SeasonalReservationTargetStatus::InReview;
            }
            SeasonalReservationTargetStatus::InReview => {
                return next == SeasonalReservationTargetStatus::Reopened
                    || next == SeasonalReservationTargetStatus::Archived;
            }
            SeasonalReservationTargetStatus::Resolved => {
                return next == SeasonalReservationTargetStatus::Reopened
                    || next == SeasonalReservationTargetStatus::Archived;
            }
            SeasonalReservationTargetStatus::Reopened => {
                return next == SeasonalReservationTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalReservationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal reservation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal reservation target", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.burst as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalReservationTarget {
        SeasonalReservationTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            rotation_days: self.rotation_days,
            name: self.name.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal reservation target in status open.
pub fn new_seasonal_reservation_target(code: &str, label: &str) -> SeasonalReservationTarget {
    SeasonalReservationTarget {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        rotation_days: 0,
        name: "".to_string(),
        approved: false,
        status: SeasonalReservationTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_reservation_target("code-314", "label-452");
        assert_eq!(item.status, SeasonalReservationTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_seasonal_reservation_target("code-314", "label-452");
        if item.move_to(SeasonalReservationTargetStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalReservationTargetStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_reservation_target("code-314", "label-452");
        item.status = SeasonalReservationTargetStatus::Archived;
        assert!(
            item.move_to(SeasonalReservationTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
