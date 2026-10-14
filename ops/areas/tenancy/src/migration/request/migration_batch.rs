use super::*;

use acme_kit::text;

/// Where a migration batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl MigrationBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for MigrationBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One migration batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationBatch {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub limit: i64,
    pub description: String,
    pub tenant_id: String,
    pub checksum: String,
    pub score: f64,
    pub capacity: i64,
    pub quantity: i64,
    pub status: MigrationBatchStatus,
}

impl MigrationBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationBatchStatus) -> bool {
        match self.status {
            MigrationBatchStatus::Draft => {
                return next == MigrationBatchStatus::Pending || next == MigrationBatchStatus::Active;
            }
            MigrationBatchStatus::Pending => {
                return next == MigrationBatchStatus::Suspended || next == MigrationBatchStatus::Closed;
            }
            MigrationBatchStatus::Active => {
                match next {
                    MigrationBatchStatus::Suspended | MigrationBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationBatchStatus::Suspended => {
                match next {
                    MigrationBatchStatus::Closed => {
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

    pub fn move_to(&mut self, next: MigrationBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quantity as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationBatch {
        MigrationBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            limit: self.limit,
            description: self.description.clone(),
            tenant_id: self.tenant_id.clone(),
            checksum: self.checksum.clone(),
            score: self.score,
            capacity: self.capacity,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a migration batch in status draft.
pub fn new_migration_batch(code: &str, name: &str) -> MigrationBatch {
    MigrationBatch {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        limit: 0,
        description: "".to_string(),
        tenant_id: "".to_string(),
        checksum: "".to_string(),
        score: 0.0,
        capacity: 0,
        quantity: 0,
        status: MigrationBatchStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_migration_batch("code-397", "name-985");
        assert_eq!(item.status, MigrationBatchStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_migration_batch("code-397", "name-985");
        if item.move_to(MigrationBatchStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationBatchStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_migration_batch("code-397", "name-985");
        item.status = MigrationBatchStatus::Closed;
        assert!(
            item.move_to(MigrationBatchStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
