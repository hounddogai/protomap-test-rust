use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending migration group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingMigrationGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingMigrationGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingMigrationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending migration group as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationGroup {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub attributes: BTreeMap<String, String>,
    pub cost: f64,
    pub description: String,
    pub status: PendingMigrationGroupStatus,
}

impl PendingMigrationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingMigrationGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingMigrationGroupStatus) -> bool {
        match self.status {
            PendingMigrationGroupStatus::Proposed => {
                return next == PendingMigrationGroupStatus::Approved || next == PendingMigrationGroupStatus::Rejected;
            }
            PendingMigrationGroupStatus::Approved => {
                return next == PendingMigrationGroupStatus::Rejected;
            }
            PendingMigrationGroupStatus::Rejected => {
                match next {
                    PendingMigrationGroupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingMigrationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending migration group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending migration group", self.code)
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
        self.cost
    }

    pub fn copy(&self) -> PendingMigrationGroup {
        PendingMigrationGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            attributes: self.attributes.clone(),
            cost: self.cost,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending migration group in status proposed.
pub fn new_pending_migration_group(code: &str, label: &str) -> PendingMigrationGroup {
    PendingMigrationGroup {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        attributes: BTreeMap::new(),
        cost: 0.0,
        description: "".to_string(),
        status: PendingMigrationGroupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_migration_group("code-529", "label-190");
        assert_eq!(item.status, PendingMigrationGroupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_pending_migration_group("code-529", "label-190");
        if item.move_to(PendingMigrationGroupStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingMigrationGroupStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_migration_group("code-529", "label-190");
        item.status = PendingMigrationGroupStatus::Withdrawn;
        assert!(
            item.move_to(PendingMigrationGroupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
