use super::*;

use acme_kit::text;

/// Where a suspension rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SuspensionRollupStatus {
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

impl std::fmt::Display for SuspensionRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRollup {
    pub code: String,
    pub title: String,
    pub host: String,
    pub phone: String,
    pub notes: String,
    pub quantity: i64,
    pub status: SuspensionRollupStatus,
}

impl SuspensionRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionRollupStatus) -> bool {
        match self.status {
            SuspensionRollupStatus::New => {
                match next {
                    SuspensionRollupStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionRollupStatus::Queued => {
                return next == SuspensionRollupStatus::Running || next == SuspensionRollupStatus::Failed;
            }
            SuspensionRollupStatus::Running => {
                return next == SuspensionRollupStatus::Done || next == SuspensionRollupStatus::Failed;
            }
            SuspensionRollupStatus::Done => {
                return next == SuspensionRollupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "suspension rollup", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> SuspensionRollup {
        SuspensionRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            phone: self.phone.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a suspension rollup in status new.
pub fn new_suspension_rollup(code: &str, title: &str) -> SuspensionRollup {
    SuspensionRollup {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        phone: "".to_string(),
        notes: "".to_string(),
        quantity: 0,
        status: SuspensionRollupStatus::New,
    }
}
