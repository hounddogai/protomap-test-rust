use super::*;

use acme_kit::text;

/// Where a scheduled zone reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledZoneReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledZoneReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledZoneReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled zone reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneReservation {
    pub code: String,
    pub label: String,
    pub name: String,
    pub category: String,
    pub grace_days: i64,
    pub fingerprint: String,
    pub timezone: String,
    pub external_ref: String,
    pub rotation_days: i64,
    pub rate: f64,
    pub status: ScheduledZoneReservationStatus,
}

impl ScheduledZoneReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledZoneReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledZoneReservationStatus) -> bool {
        match self.status {
            ScheduledZoneReservationStatus::Created => {
                return next == ScheduledZoneReservationStatus::Retired;
            }
            ScheduledZoneReservationStatus::Validated => {
                match next {
                    ScheduledZoneReservationStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledZoneReservationStatus::Published => {
                match next {
                    ScheduledZoneReservationStatus::Retired => {
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

    pub fn move_to(&mut self, next: ScheduledZoneReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled zone reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.rotation_days as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledZoneReservation {
        ScheduledZoneReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            grace_days: self.grace_days,
            fingerprint: self.fingerprint.clone(),
            timezone: self.timezone.clone(),
            external_ref: self.external_ref.clone(),
            rotation_days: self.rotation_days,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a scheduled zone reservation in status created.
pub fn new_scheduled_zone_reservation(code: &str, label: &str) -> ScheduledZoneReservation {
    ScheduledZoneReservation {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        category: "".to_string(),
        grace_days: 0,
        fingerprint: "".to_string(),
        timezone: "".to_string(),
        external_ref: "".to_string(),
        rotation_days: 0,
        rate: 0.0,
        status: ScheduledZoneReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_zone_reservation("code-223", "label-474");
        assert_eq!(item.status, ScheduledZoneReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_scheduled_zone_reservation("code-223", "label-474");
        if item.move_to(ScheduledZoneReservationStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledZoneReservationStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_zone_reservation("code-223", "label-474");
        item.status = ScheduledZoneReservationStatus::Retired;
        assert!(
            item.move_to(ScheduledZoneReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
