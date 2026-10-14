use super::*;

use acme_kit::text;

/// Where a runbook allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RunbookAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RunbookAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook allocation as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookAllocation {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub description: String,
    pub cost: f64,
    pub visible: bool,
    pub label: String,
    pub starts_at: i64,
    pub timezone: String,
    pub status: RunbookAllocationStatus,
}

impl RunbookAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookAllocationStatus) -> bool {
        match self.status {
            RunbookAllocationStatus::Proposed => {
                return next == RunbookAllocationStatus::Rejected;
            }
            RunbookAllocationStatus::Approved => {
                return next == RunbookAllocationStatus::Rejected || next == RunbookAllocationStatus::Withdrawn;
            }
            RunbookAllocationStatus::Rejected => {
                match next {
                    RunbookAllocationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RunbookAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.cost - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookAllocation {
        RunbookAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            description: self.description.clone(),
            cost: self.cost,
            visible: self.visible,
            label: self.label.clone(),
            starts_at: self.starts_at,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a runbook allocation in status proposed.
pub fn new_runbook_allocation(code: &str, name: &str) -> RunbookAllocation {
    RunbookAllocation {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        description: "".to_string(),
        cost: 0.0,
        visible: false,
        label: "".to_string(),
        starts_at: 0,
        timezone: "".to_string(),
        status: RunbookAllocationStatus::Proposed,
    }
}
