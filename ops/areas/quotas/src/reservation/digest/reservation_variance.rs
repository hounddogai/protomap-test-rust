use super::*;

use acme_kit::text;

/// Where a reservation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReservationVarianceStatus {
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

impl std::fmt::Display for ReservationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation variance as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationVariance {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub quota_units: i64,
    pub capacity: i64,
    pub country: String,
    pub status: ReservationVarianceStatus,
}

impl ReservationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationVarianceStatus) -> bool {
        match self.status {
            ReservationVarianceStatus::Open => {
                match next {
                    ReservationVarianceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationVarianceStatus::InReview => {
                match next {
                    ReservationVarianceStatus::Resolved | ReservationVarianceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationVarianceStatus::Resolved => {
                return next == ReservationVarianceStatus::Reopened || next == ReservationVarianceStatus::Archived;
            }
            ReservationVarianceStatus::Reopened => {
                return next == ReservationVarianceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "reservation variance", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.capacity as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationVariance {
        ReservationVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            quota_units: self.quota_units,
            capacity: self.capacity,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a reservation variance in status open.
pub fn new_reservation_variance(code: &str, label: &str) -> ReservationVariance {
    ReservationVariance {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        quota_units: 0,
        capacity: 0,
        country: "".to_string(),
        status: ReservationVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_reservation_variance("code-375", "label-251");
        assert_eq!(item.status, ReservationVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_reservation_variance("code-375", "label-251");
        if item.move_to(ReservationVarianceStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationVarianceStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_reservation_variance("code-375", "label-251");
        item.status = ReservationVarianceStatus::Archived;
        assert!(
            item.move_to(ReservationVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
