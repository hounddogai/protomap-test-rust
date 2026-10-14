use super::*;

use acme_kit::text;

/// Where a default migration allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultMigrationAllocationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultMigrationAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for DefaultMigrationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default migration allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationAllocation {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub channel: String,
    pub namespace: String,
    pub host: String,
    pub version: i64,
    pub status: DefaultMigrationAllocationStatus,
}

impl DefaultMigrationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultMigrationAllocationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultMigrationAllocationStatus) -> bool {
        match self.status {
            DefaultMigrationAllocationStatus::New => {
                match next {
                    DefaultMigrationAllocationStatus::Running | DefaultMigrationAllocationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMigrationAllocationStatus::Queued => {
                return next == DefaultMigrationAllocationStatus::Done
                    || next == DefaultMigrationAllocationStatus::Failed;
            }
            DefaultMigrationAllocationStatus::Running => {
                return next == DefaultMigrationAllocationStatus::Done
                    || next == DefaultMigrationAllocationStatus::Failed;
            }
            DefaultMigrationAllocationStatus::Done => {
                return next == DefaultMigrationAllocationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultMigrationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default migration allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.version as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultMigrationAllocation {
        DefaultMigrationAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            channel: self.channel.clone(),
            namespace: self.namespace.clone(),
            host: self.host.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a default migration allocation in status new.
pub fn new_default_migration_allocation(code: &str, label: &str) -> DefaultMigrationAllocation {
    DefaultMigrationAllocation {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        channel: "".to_string(),
        namespace: "".to_string(),
        host: "".to_string(),
        version: 0,
        status: DefaultMigrationAllocationStatus::New,
    }
}
