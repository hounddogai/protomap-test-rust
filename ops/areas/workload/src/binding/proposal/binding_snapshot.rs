use super::*;

use acme_kit::text;

/// Where a binding snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingSnapshotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BindingSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BindingSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A binding snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingSnapshot {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub issuer: String,
    pub external_ref: String,
    pub status: BindingSnapshotStatus,
}

impl BindingSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingSnapshotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingSnapshotStatus) -> bool {
        match self.status {
            BindingSnapshotStatus::Proposed => {
                match next {
                    BindingSnapshotStatus::Rejected | BindingSnapshotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingSnapshotStatus::Approved => {
                return next == BindingSnapshotStatus::Withdrawn;
            }
            BindingSnapshotStatus::Rejected => {
                return next == BindingSnapshotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> BindingSnapshot {
        BindingSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            issuer: self.issuer.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a binding snapshot in status proposed.
pub fn new_binding_snapshot(code: &str, label: &str) -> BindingSnapshot {
    BindingSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        issuer: "".to_string(),
        external_ref: "".to_string(),
        status: BindingSnapshotStatus::Proposed,
    }
}
