use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched migration rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedMigrationRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedMigrationRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedMigrationRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched migration rule as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRule {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub attributes: BTreeMap<String, String>,
    pub floor_value: f64,
    pub quota_units: i64,
    pub status: BatchedMigrationRuleStatus,
}

impl BatchedMigrationRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedMigrationRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedMigrationRuleStatus) -> bool {
        match self.status {
            BatchedMigrationRuleStatus::Requested => {
                match next {
                    BatchedMigrationRuleStatus::Granted | BatchedMigrationRuleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedMigrationRuleStatus::Granted => {
                match next {
                    BatchedMigrationRuleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedMigrationRuleStatus::Revoked => {
                return next == BatchedMigrationRuleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedMigrationRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched migration rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
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

    pub fn projected_volume(&self) -> f64 {
        ((self.quota_units as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedMigrationRule {
        BatchedMigrationRule {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            attributes: self.attributes.clone(),
            floor_value: self.floor_value,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched migration rule in status requested.
pub fn new_batched_migration_rule(code: &str, name: &str) -> BatchedMigrationRule {
    BatchedMigrationRule {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        attributes: BTreeMap::new(),
        floor_value: 0.0,
        quota_units: 0,
        status: BatchedMigrationRuleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_migration_rule("code-488", "name-926");
        assert_eq!(item.status, BatchedMigrationRuleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_batched_migration_rule("code-488", "name-926");
        if item.move_to(BatchedMigrationRuleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedMigrationRuleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_migration_rule("code-488", "name-926");
        item.status = BatchedMigrationRuleStatus::Expired;
        assert!(
            item.move_to(BatchedMigrationRuleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
