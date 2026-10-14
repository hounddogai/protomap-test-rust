use super::*;

use acme_kit::text;

/// Where a pending plan entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPlanEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingPlanEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingPlanEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending plan entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanEntry {
    pub code: String,
    pub label: String,
    pub source: String,
    pub region: String,
    pub minimum: f64,
    pub status: PendingPlanEntryStatus,
}

impl PendingPlanEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPlanEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPlanEntryStatus) -> bool {
        match self.status {
            PendingPlanEntryStatus::Proposed => {
                return next == PendingPlanEntryStatus::Rejected || next == PendingPlanEntryStatus::Withdrawn;
            }
            PendingPlanEntryStatus::Approved => {
                match next {
                    PendingPlanEntryStatus::Rejected | PendingPlanEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPlanEntryStatus::Rejected => {
                return next == PendingPlanEntryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingPlanEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending plan entry {} cannot move from {} to {}",
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
        self.minimum
    }

    pub fn copy(&self) -> PendingPlanEntry {
        PendingPlanEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            region: self.region.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending plan entry in status proposed.
pub fn new_pending_plan_entry(code: &str, label: &str) -> PendingPlanEntry {
    PendingPlanEntry {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        region: "".to_string(),
        minimum: 0.0,
        status: PendingPlanEntryStatus::Proposed,
    }
}
