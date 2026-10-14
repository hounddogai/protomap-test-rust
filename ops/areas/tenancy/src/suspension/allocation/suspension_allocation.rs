use super::*;

use acme_kit::text;

/// Where a suspension allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SuspensionAllocationStatus {
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

impl std::fmt::Display for SuspensionAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a suspension allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAllocation {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub principal: String,
    pub active: bool,
    pub name: String,
    pub title: String,
    pub fingerprint: String,
    pub amount: f64,
    pub status: SuspensionAllocationStatus,
}

impl SuspensionAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionAllocationStatus) -> bool {
        match self.status {
            SuspensionAllocationStatus::Planned => {
                match next {
                    SuspensionAllocationStatus::Completed | SuspensionAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionAllocationStatus::Scheduled => {
                match next {
                    SuspensionAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionAllocationStatus::InProgress => {
                return next == SuspensionAllocationStatus::Completed || next == SuspensionAllocationStatus::Cancelled;
            }
            SuspensionAllocationStatus::Completed => {
                return next == SuspensionAllocationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> SuspensionAllocation {
        SuspensionAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            principal: self.principal.clone(),
            active: self.active,
            name: self.name.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a suspension allocation in status planned.
pub fn new_suspension_allocation(code: &str, label: &str) -> SuspensionAllocation {
    SuspensionAllocation {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        principal: "".to_string(),
        active: false,
        name: "".to_string(),
        title: "".to_string(),
        fingerprint: "".to_string(),
        amount: 0.0,
        status: SuspensionAllocationStatus::Planned,
    }
}
