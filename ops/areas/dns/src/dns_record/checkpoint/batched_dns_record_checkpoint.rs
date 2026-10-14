use super::*;

use acme_kit::text;

/// Where a batched dns record checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedDnsRecordCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedDnsRecordCheckpointStatus {
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

impl std::fmt::Display for BatchedDnsRecordCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched dns record checkpoint as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordCheckpoint {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub visible: bool,
    pub active: bool,
    pub owner_id: String,
    pub score: f64,
    pub revision: i64,
    pub status: BatchedDnsRecordCheckpointStatus,
}

impl BatchedDnsRecordCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedDnsRecordCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedDnsRecordCheckpointStatus) -> bool {
        match self.status {
            BatchedDnsRecordCheckpointStatus::Draft => {
                return next == BatchedDnsRecordCheckpointStatus::Suspended;
            }
            BatchedDnsRecordCheckpointStatus::Pending => {
                return next == BatchedDnsRecordCheckpointStatus::Active
                    || next == BatchedDnsRecordCheckpointStatus::Suspended;
            }
            BatchedDnsRecordCheckpointStatus::Active => {
                match next {
                    BatchedDnsRecordCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedDnsRecordCheckpointStatus::Suspended => {
                return next == BatchedDnsRecordCheckpointStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedDnsRecordCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched dns record checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched dns record checkpoint", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.score - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedDnsRecordCheckpoint {
        BatchedDnsRecordCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            visible: self.visible,
            active: self.active,
            owner_id: self.owner_id.clone(),
            score: self.score,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched dns record checkpoint in status draft.
pub fn new_batched_dns_record_checkpoint(code: &str, label: &str) -> BatchedDnsRecordCheckpoint {
    BatchedDnsRecordCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        visible: false,
        active: false,
        owner_id: "".to_string(),
        score: 0.0,
        revision: 0,
        status: BatchedDnsRecordCheckpointStatus::Draft,
    }
}
