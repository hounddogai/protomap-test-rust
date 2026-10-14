use super::*;

use acme_kit::text;

/// Where a scheduled binding allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledBindingAllocationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledBindingAllocationStatus {
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

impl std::fmt::Display for ScheduledBindingAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled binding allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingAllocation {
    pub code: String,
    pub label: String,
    pub description: String,
    pub approved: bool,
    pub quota_units: i64,
    pub phone: String,
    pub quantity: i64,
    pub segment: String,
    pub status: ScheduledBindingAllocationStatus,
}

impl ScheduledBindingAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledBindingAllocationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledBindingAllocationStatus) -> bool {
        match self.status {
            ScheduledBindingAllocationStatus::Draft => {
                return next == ScheduledBindingAllocationStatus::Pending
                    || next == ScheduledBindingAllocationStatus::Active;
            }
            ScheduledBindingAllocationStatus::Pending => {
                return next == ScheduledBindingAllocationStatus::Suspended;
            }
            ScheduledBindingAllocationStatus::Active => {
                return next == ScheduledBindingAllocationStatus::Suspended
                    || next == ScheduledBindingAllocationStatus::Closed;
            }
            ScheduledBindingAllocationStatus::Suspended => {
                match next {
                    ScheduledBindingAllocationStatus::Closed => {
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

    pub fn move_to(&mut self, next: ScheduledBindingAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled binding allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled binding allocation", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.quantity as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledBindingAllocation {
        ScheduledBindingAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            approved: self.approved,
            quota_units: self.quota_units,
            phone: self.phone.clone(),
            quantity: self.quantity,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled binding allocation in status draft.
pub fn new_scheduled_binding_allocation(code: &str, label: &str) -> ScheduledBindingAllocation {
    ScheduledBindingAllocation {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        approved: false,
        quota_units: 0,
        phone: "".to_string(),
        quantity: 0,
        segment: "".to_string(),
        status: ScheduledBindingAllocationStatus::Draft,
    }
}
