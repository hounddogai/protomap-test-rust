use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched approval checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedApprovalCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedApprovalCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedApprovalCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched approval checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalCheckpoint {
    pub code: String,
    pub name: String,
    pub country: String,
    pub capacity: i64,
    pub checksum: String,
    pub attributes: BTreeMap<String, String>,
    pub visible: bool,
    pub label: String,
    pub title: String,
    pub status: BatchedApprovalCheckpointStatus,
}

impl BatchedApprovalCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedApprovalCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedApprovalCheckpointStatus) -> bool {
        match self.status {
            BatchedApprovalCheckpointStatus::Created => {
                return next == BatchedApprovalCheckpointStatus::Validated
                    || next == BatchedApprovalCheckpointStatus::Retired;
            }
            BatchedApprovalCheckpointStatus::Validated => {
                return next == BatchedApprovalCheckpointStatus::Published
                    || next == BatchedApprovalCheckpointStatus::Retired;
            }
            BatchedApprovalCheckpointStatus::Published => {
                return next == BatchedApprovalCheckpointStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedApprovalCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched approval checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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
        self.capacity as f64
    }

    pub fn copy(&self) -> BatchedApprovalCheckpoint {
        BatchedApprovalCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            capacity: self.capacity,
            checksum: self.checksum.clone(),
            attributes: self.attributes.clone(),
            visible: self.visible,
            label: self.label.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a batched approval checkpoint in status created.
pub fn new_batched_approval_checkpoint(code: &str, name: &str) -> BatchedApprovalCheckpoint {
    BatchedApprovalCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        capacity: 0,
        checksum: "".to_string(),
        attributes: BTreeMap::new(),
        visible: false,
        label: "".to_string(),
        title: "".to_string(),
        status: BatchedApprovalCheckpointStatus::Created,
    }
}
