use super::*;

use acme_kit::text;

/// Where a migration ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MigrationLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MigrationLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration ledger as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationLedger {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub budget: f64,
    pub country: String,
    pub audience: String,
    pub title: String,
    pub discount: f64,
    pub cipher: String,
    pub unit: String,
    pub floor_value: f64,
    pub status: MigrationLedgerStatus,
}

impl MigrationLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationLedgerStatus) -> bool {
        match self.status {
            MigrationLedgerStatus::Created => {
                match next {
                    MigrationLedgerStatus::Validated | MigrationLedgerStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationLedgerStatus::Validated => {
                return next == MigrationLedgerStatus::Published || next == MigrationLedgerStatus::Retired;
            }
            MigrationLedgerStatus::Published => {
                match next {
                    MigrationLedgerStatus::Retired => {
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

    pub fn move_to(&mut self, next: MigrationLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        (self.floor_value * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationLedger {
        MigrationLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            budget: self.budget,
            country: self.country.clone(),
            audience: self.audience.clone(),
            title: self.title.clone(),
            discount: self.discount,
            cipher: self.cipher.clone(),
            unit: self.unit.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a migration ledger in status created.
pub fn new_migration_ledger(code: &str, label: &str) -> MigrationLedger {
    MigrationLedger {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        budget: 0.0,
        country: "".to_string(),
        audience: "".to_string(),
        title: "".to_string(),
        discount: 0.0,
        cipher: "".to_string(),
        unit: "".to_string(),
        floor_value: 0.0,
        status: MigrationLedgerStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_migration_ledger("code-914", "label-707");
        assert_eq!(item.status, MigrationLedgerStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_migration_ledger("code-914", "label-707");
        if item.move_to(MigrationLedgerStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationLedgerStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_migration_ledger("code-914", "label-707");
        item.status = MigrationLedgerStatus::Retired;
        assert!(
            item.move_to(MigrationLedgerStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
