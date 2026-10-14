use super::*;

use acme_kit::text;

/// Where a plan cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanCycleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PlanCycleStatus {
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

impl std::fmt::Display for PlanCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a plan cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanCycle {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub checksum: String,
    pub source: String,
    pub host: String,
    pub minimum: f64,
    pub currency: String,
    pub label: String,
    pub owner_id: String,
    pub status: PlanCycleStatus,
}

impl PlanCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanCycleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanCycleStatus) -> bool {
        match self.status {
            PlanCycleStatus::Planned => {
                return next == PlanCycleStatus::Scheduled || next == PlanCycleStatus::InProgress;
            }
            PlanCycleStatus::Scheduled => {
                match next {
                    PlanCycleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanCycleStatus::InProgress => {
                match next {
                    PlanCycleStatus::Completed | PlanCycleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanCycleStatus::Completed => {
                return next == PlanCycleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.minimum
    }

    pub fn copy(&self) -> PlanCycle {
        PlanCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            checksum: self.checksum.clone(),
            source: self.source.clone(),
            host: self.host.clone(),
            minimum: self.minimum,
            currency: self.currency.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a plan cycle in status planned.
pub fn new_plan_cycle(code: &str, title: &str) -> PlanCycle {
    PlanCycle {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        checksum: "".to_string(),
        source: "".to_string(),
        host: "".to_string(),
        minimum: 0.0,
        currency: "".to_string(),
        label: "".to_string(),
        owner_id: "".to_string(),
        status: PlanCycleStatus::Planned,
    }
}
