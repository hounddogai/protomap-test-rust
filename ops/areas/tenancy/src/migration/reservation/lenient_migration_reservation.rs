use super::*;

use acme_kit::text;

/// Where a lenient migration reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientMigrationReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientMigrationReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientMigrationReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient migration reservation as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMigrationReservation {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub channel: String,
    pub grace_days: i64,
    pub priority: i64,
    pub version: i64,
    pub burst: i64,
    pub owner_id: String,
    pub quota_units: i64,
    pub region: String,
    pub status: LenientMigrationReservationStatus,
}

impl LenientMigrationReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientMigrationReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientMigrationReservationStatus) -> bool {
        match self.status {
            LenientMigrationReservationStatus::Proposed => {
                match next {
                    LenientMigrationReservationStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientMigrationReservationStatus::Approved => {
                return next == LenientMigrationReservationStatus::Withdrawn;
            }
            LenientMigrationReservationStatus::Rejected => {
                match next {
                    LenientMigrationReservationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LenientMigrationReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient migration reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient migration reservation", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.priority as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientMigrationReservation {
        LenientMigrationReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            channel: self.channel.clone(),
            grace_days: self.grace_days,
            priority: self.priority,
            version: self.version,
            burst: self.burst,
            owner_id: self.owner_id.clone(),
            quota_units: self.quota_units,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a lenient migration reservation in status proposed.
pub fn new_lenient_migration_reservation(code: &str, label: &str) -> LenientMigrationReservation {
    LenientMigrationReservation {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        channel: "".to_string(),
        grace_days: 0,
        priority: 0,
        version: 0,
        burst: 0,
        owner_id: "".to_string(),
        quota_units: 0,
        region: "".to_string(),
        status: LenientMigrationReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_migration_reservation("code-877", "label-698");
        assert_eq!(item.status, LenientMigrationReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_lenient_migration_reservation("code-877", "label-698");
        if item.move_to(LenientMigrationReservationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientMigrationReservationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_migration_reservation("code-877", "label-698");
        item.status = LenientMigrationReservationStatus::Withdrawn;
        assert!(
            item.move_to(LenientMigrationReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
