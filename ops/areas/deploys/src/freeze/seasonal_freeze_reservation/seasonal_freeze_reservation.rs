use super::*;

use acme_kit::text;

/// Where a seasonal freeze reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFreezeReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalFreezeReservationStatus {
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

impl std::fmt::Display for SeasonalFreezeReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal freeze reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeReservation {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub channel: String,
    pub window_days: i64,
    pub priority: i64,
    pub ttl_seconds: i64,
    pub amount: f64,
    pub category: String,
    pub batch_size: i64,
    pub status: SeasonalFreezeReservationStatus,
}

impl SeasonalFreezeReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFreezeReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFreezeReservationStatus) -> bool {
        match self.status {
            SeasonalFreezeReservationStatus::Open => {
                return next == SeasonalFreezeReservationStatus::Resolved;
            }
            SeasonalFreezeReservationStatus::InReview => {
                return next == SeasonalFreezeReservationStatus::Resolved;
            }
            SeasonalFreezeReservationStatus::Resolved => {
                return next == SeasonalFreezeReservationStatus::Reopened
                    || next == SeasonalFreezeReservationStatus::Archived;
            }
            SeasonalFreezeReservationStatus::Reopened => {
                return next == SeasonalFreezeReservationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalFreezeReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal freeze reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "seasonal freeze reservation", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.duration_minutes as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalFreezeReservation {
        SeasonalFreezeReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            channel: self.channel.clone(),
            window_days: self.window_days,
            priority: self.priority,
            ttl_seconds: self.ttl_seconds,
            amount: self.amount,
            category: self.category.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a seasonal freeze reservation in status open.
pub fn new_seasonal_freeze_reservation(code: &str, label: &str) -> SeasonalFreezeReservation {
    SeasonalFreezeReservation {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        channel: "".to_string(),
        window_days: 0,
        priority: 0,
        ttl_seconds: 0,
        amount: 0.0,
        category: "".to_string(),
        batch_size: 0,
        status: SeasonalFreezeReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_freeze_reservation("code-138", "label-55");
        assert_eq!(item.status, SeasonalFreezeReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_seasonal_freeze_reservation("code-138", "label-55");
        if item.move_to(SeasonalFreezeReservationStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalFreezeReservationStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_freeze_reservation("code-138", "label-55");
        item.status = SeasonalFreezeReservationStatus::Archived;
        assert!(
            item.move_to(SeasonalFreezeReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
