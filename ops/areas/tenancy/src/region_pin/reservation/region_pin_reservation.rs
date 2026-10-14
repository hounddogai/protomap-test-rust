use super::*;

use acme_kit::text;

/// Where a region pin reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionPinReservationStatus {
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

impl std::fmt::Display for RegionPinReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinReservation {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub name: String,
    pub quota_units: i64,
    pub pinned: bool,
    pub cipher: String,
    pub ttl_seconds: i64,
    pub archived: bool,
    pub fingerprint: String,
    pub audience: String,
    pub status: RegionPinReservationStatus,
}

impl RegionPinReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinReservationStatus) -> bool {
        match self.status {
            RegionPinReservationStatus::Planned => {
                return next == RegionPinReservationStatus::InProgress;
            }
            RegionPinReservationStatus::Scheduled => {
                return next == RegionPinReservationStatus::Completed;
            }
            RegionPinReservationStatus::InProgress => {
                match next {
                    RegionPinReservationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinReservationStatus::Completed => {
                return next == RegionPinReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "region pin reservation", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinReservation {
        RegionPinReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            name: self.name.clone(),
            quota_units: self.quota_units,
            pinned: self.pinned,
            cipher: self.cipher.clone(),
            ttl_seconds: self.ttl_seconds,
            archived: self.archived,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a region pin reservation in status planned.
pub fn new_region_pin_reservation(code: &str, label: &str) -> RegionPinReservation {
    RegionPinReservation {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        name: "".to_string(),
        quota_units: 0,
        pinned: false,
        cipher: "".to_string(),
        ttl_seconds: 0,
        archived: false,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        status: RegionPinReservationStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_region_pin_reservation("code-491", "label-108");
        assert_eq!(item.status, RegionPinReservationStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_region_pin_reservation("code-491", "label-108");
        if item.move_to(RegionPinReservationStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinReservationStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_region_pin_reservation("code-491", "label-108");
        item.status = RegionPinReservationStatus::Cancelled;
        assert!(
            item.move_to(RegionPinReservationStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
