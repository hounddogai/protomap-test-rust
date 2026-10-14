use super::*;

use acme_kit::text;

/// Where a batched migration record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedMigrationRecordStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedMigrationRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedMigrationRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched migration record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRecord {
    pub code: String,
    pub label: String,
    pub country: String,
    pub unit: String,
    pub checksum: String,
    pub channel: String,
    pub amount: f64,
    pub status: BatchedMigrationRecordStatus,
}

impl BatchedMigrationRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedMigrationRecordStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedMigrationRecordStatus) -> bool {
        match self.status {
            BatchedMigrationRecordStatus::Idle => {
                match next {
                    BatchedMigrationRecordStatus::Draining | BatchedMigrationRecordStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedMigrationRecordStatus::Warming => {
                match next {
                    BatchedMigrationRecordStatus::Ready | BatchedMigrationRecordStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedMigrationRecordStatus::Ready => {
                match next {
                    BatchedMigrationRecordStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedMigrationRecordStatus::Draining => {
                return next == BatchedMigrationRecordStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedMigrationRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched migration record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched migration record", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> BatchedMigrationRecord {
        BatchedMigrationRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            unit: self.unit.clone(),
            checksum: self.checksum.clone(),
            channel: self.channel.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a batched migration record in status idle.
pub fn new_batched_migration_record(code: &str, label: &str) -> BatchedMigrationRecord {
    BatchedMigrationRecord {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        unit: "".to_string(),
        checksum: "".to_string(),
        channel: "".to_string(),
        amount: 0.0,
        status: BatchedMigrationRecordStatus::Idle,
    }
}
