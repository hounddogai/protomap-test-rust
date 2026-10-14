use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a reservation snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ReservationSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ReservationSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSnapshot {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub attributes: BTreeMap<String, String>,
    pub title: String,
    pub quantity: i64,
    pub status: ReservationSnapshotStatus,
}

impl ReservationSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationSnapshotStatus) -> bool {
        match self.status {
            ReservationSnapshotStatus::Created => {
                return next == ReservationSnapshotStatus::Validated || next == ReservationSnapshotStatus::Retired;
            }
            ReservationSnapshotStatus::Validated => {
                return next == ReservationSnapshotStatus::Published || next == ReservationSnapshotStatus::Retired;
            }
            ReservationSnapshotStatus::Published => {
                match next {
                    ReservationSnapshotStatus::Retired => {
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

    pub fn move_to(&mut self, next: ReservationSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ReservationSnapshot {
        ReservationSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            attributes: self.attributes.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a reservation snapshot in status created.
pub fn new_reservation_snapshot(code: &str, name: &str) -> ReservationSnapshot {
    ReservationSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        attributes: BTreeMap::new(),
        title: "".to_string(),
        quantity: 0,
        status: ReservationSnapshotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_reservation_snapshot("code-161", "name-478");
        assert_eq!(item.status, ReservationSnapshotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_reservation_snapshot("code-161", "name-478");
        if item.move_to(ReservationSnapshotStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationSnapshotStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_reservation_snapshot("code-161", "name-478");
        item.status = ReservationSnapshotStatus::Retired;
        assert!(
            item.move_to(ReservationSnapshotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
