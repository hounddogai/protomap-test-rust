use super::*;

use acme_kit::text;

/// Where a migration reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl MigrationReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for MigrationReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration reservation as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReservation {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub ttl_seconds: i64,
    pub unit: String,
    pub budget: f64,
    pub status: MigrationReservationStatus,
}

impl MigrationReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationReservationStatus) -> bool {
        match self.status {
            MigrationReservationStatus::Proposed => {
                match next {
                    MigrationReservationStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationReservationStatus::Approved => {
                match next {
                    MigrationReservationStatus::Rejected | MigrationReservationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationReservationStatus::Rejected => {
                match next {
                    MigrationReservationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: MigrationReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.window_days as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationReservation {
        MigrationReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            ttl_seconds: self.ttl_seconds,
            unit: self.unit.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a migration reservation in status proposed.
pub fn new_migration_reservation(code: &str, label: &str) -> MigrationReservation {
    MigrationReservation {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        ttl_seconds: 0,
        unit: "".to_string(),
        budget: 0.0,
        status: MigrationReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_migration_reservation("code-242", "label-506");
        assert_eq!(item.status, MigrationReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_migration_reservation("code-242", "label-506");
        if item.move_to(MigrationReservationStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationReservationStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_migration_reservation("code-242", "label-506");
        item.status = MigrationReservationStatus::Withdrawn;
        assert!(
            item.move_to(MigrationReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
