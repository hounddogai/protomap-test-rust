use super::*;

use acme_kit::text;

/// Where a reservation adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReservationAdjustmentStatus {
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

impl std::fmt::Display for ReservationAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a reservation adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAdjustment {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub quota_units: i64,
    pub label: String,
    pub checksum: String,
    pub active: bool,
    pub expires_at: i64,
    pub duration_minutes: i64,
    pub status: ReservationAdjustmentStatus,
}

impl ReservationAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationAdjustmentStatus) -> bool {
        match self.status {
            ReservationAdjustmentStatus::Open => {
                match next {
                    ReservationAdjustmentStatus::InReview | ReservationAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationAdjustmentStatus::InReview => {
                match next {
                    ReservationAdjustmentStatus::Resolved | ReservationAdjustmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationAdjustmentStatus::Resolved => {
                match next {
                    ReservationAdjustmentStatus::Reopened | ReservationAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationAdjustmentStatus::Reopened => {
                match next {
                    ReservationAdjustmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: ReservationAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation adjustment {} cannot move from {} to {}",
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
        format!("{} {}", "reservation adjustment", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationAdjustment {
        ReservationAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            quota_units: self.quota_units,
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            active: self.active,
            expires_at: self.expires_at,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a reservation adjustment in status open.
pub fn new_reservation_adjustment(code: &str, title: &str) -> ReservationAdjustment {
    ReservationAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        quota_units: 0,
        label: "".to_string(),
        checksum: "".to_string(),
        active: false,
        expires_at: 0,
        duration_minutes: 0,
        status: ReservationAdjustmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_reservation_adjustment("code-752", "title-231");
        assert_eq!(item.status, ReservationAdjustmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_reservation_adjustment("code-752", "title-231");
        if item.move_to(ReservationAdjustmentStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationAdjustmentStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_reservation_adjustment("code-752", "title-231");
        item.status = ReservationAdjustmentStatus::Archived;
        assert!(
            item.move_to(ReservationAdjustmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
