use super::*;

use acme_kit::text;

/// Where a global redaction budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRedactionBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalRedactionBudgetStatus {
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

impl std::fmt::Display for GlobalRedactionBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global redaction budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionBudget {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub reviewer: String,
    pub ceiling: f64,
    pub region: String,
    pub status: GlobalRedactionBudgetStatus,
}

impl GlobalRedactionBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRedactionBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRedactionBudgetStatus) -> bool {
        match self.status {
            GlobalRedactionBudgetStatus::Planned => {
                return next == GlobalRedactionBudgetStatus::Completed
                    || next == GlobalRedactionBudgetStatus::Cancelled;
            }
            GlobalRedactionBudgetStatus::Scheduled => {
                return next == GlobalRedactionBudgetStatus::Completed;
            }
            GlobalRedactionBudgetStatus::InProgress => {
                match next {
                    GlobalRedactionBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRedactionBudgetStatus::Completed => {
                match next {
                    GlobalRedactionBudgetStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: GlobalRedactionBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global redaction budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> GlobalRedactionBudget {
        GlobalRedactionBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            reviewer: self.reviewer.clone(),
            ceiling: self.ceiling,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a global redaction budget in status planned.
pub fn new_global_redaction_budget(code: &str, label: &str) -> GlobalRedactionBudget {
    GlobalRedactionBudget {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        reviewer: "".to_string(),
        ceiling: 0.0,
        region: "".to_string(),
        status: GlobalRedactionBudgetStatus::Planned,
    }
}
