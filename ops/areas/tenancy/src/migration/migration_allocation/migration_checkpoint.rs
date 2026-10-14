use super::*;

use acme_kit::text;

/// Where a migration checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl MigrationCheckpointStatus {
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

impl std::fmt::Display for MigrationCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCheckpoint {
    pub code: String,
    pub title: String,
    pub country: String,
    pub cluster: String,
    pub key_id: String,
    pub discount: f64,
    pub scope: String,
    pub amount: f64,
    pub tenant_id: String,
    pub maximum: f64,
    pub status: MigrationCheckpointStatus,
}

impl MigrationCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationCheckpointStatus) -> bool {
        match self.status {
            MigrationCheckpointStatus::Planned => {
                return next == MigrationCheckpointStatus::InProgress || next == MigrationCheckpointStatus::Cancelled;
            }
            MigrationCheckpointStatus::Scheduled => {
                return next == MigrationCheckpointStatus::InProgress || next == MigrationCheckpointStatus::Completed;
            }
            MigrationCheckpointStatus::InProgress => {
                match next {
                    MigrationCheckpointStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationCheckpointStatus::Completed => {
                return next == MigrationCheckpointStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.discount + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationCheckpoint {
        MigrationCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            cluster: self.cluster.clone(),
            key_id: self.key_id.clone(),
            discount: self.discount,
            scope: self.scope.clone(),
            amount: self.amount,
            tenant_id: self.tenant_id.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a migration checkpoint in status planned.
pub fn new_migration_checkpoint(code: &str, title: &str) -> MigrationCheckpoint {
    MigrationCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        cluster: "".to_string(),
        key_id: "".to_string(),
        discount: 0.0,
        scope: "".to_string(),
        amount: 0.0,
        tenant_id: "".to_string(),
        maximum: 0.0,
        status: MigrationCheckpointStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_migration_checkpoint("code-837", "title-312");
        assert_eq!(item.status, MigrationCheckpointStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_migration_checkpoint("code-837", "title-312");
        if item.move_to(MigrationCheckpointStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationCheckpointStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_migration_checkpoint("code-837", "title-312");
        item.status = MigrationCheckpointStatus::Cancelled;
        assert!(
            item.move_to(MigrationCheckpointStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
