use super::*;

use acme_kit::text;

/// Where a primary credential batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryCredentialBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryCredentialBatchStatus {
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

impl std::fmt::Display for PrimaryCredentialBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary credential batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCredentialBatch {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub owner_id: String,
    pub checksum: String,
    pub priority: i64,
    pub pinned: bool,
    pub status: PrimaryCredentialBatchStatus,
}

impl PrimaryCredentialBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryCredentialBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryCredentialBatchStatus) -> bool {
        match self.status {
            PrimaryCredentialBatchStatus::New => {
                match next {
                    PrimaryCredentialBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryCredentialBatchStatus::Queued => {
                return next == PrimaryCredentialBatchStatus::Running || next == PrimaryCredentialBatchStatus::Failed;
            }
            PrimaryCredentialBatchStatus::Running => {
                match next {
                    PrimaryCredentialBatchStatus::Done | PrimaryCredentialBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryCredentialBatchStatus::Done => {
                match next {
                    PrimaryCredentialBatchStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimaryCredentialBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary credential batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> PrimaryCredentialBatch {
        PrimaryCredentialBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            owner_id: self.owner_id.clone(),
            checksum: self.checksum.clone(),
            priority: self.priority,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary credential batch in status new.
pub fn new_primary_credential_batch(code: &str, label: &str) -> PrimaryCredentialBatch {
    PrimaryCredentialBatch {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        owner_id: "".to_string(),
        checksum: "".to_string(),
        priority: 0,
        pinned: false,
        status: PrimaryCredentialBatchStatus::New,
    }
}
