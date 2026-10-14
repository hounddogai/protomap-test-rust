use super::*;

use acme_kit::text;

/// Where a runbook entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RunbookEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RunbookEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEntry {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub retries: i64,
    pub country: String,
    pub budget: f64,
    pub principal: String,
    pub namespace: String,
    pub status: RunbookEntryStatus,
}

impl RunbookEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookEntryStatus) -> bool {
        match self.status {
            RunbookEntryStatus::Created => {
                return next == RunbookEntryStatus::Retired;
            }
            RunbookEntryStatus::Validated => {
                return next == RunbookEntryStatus::Published;
            }
            RunbookEntryStatus::Published => {
                match next {
                    RunbookEntryStatus::Retired => {
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

    pub fn move_to(&mut self, next: RunbookEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.sequence as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookEntry {
        RunbookEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            retries: self.retries,
            country: self.country.clone(),
            budget: self.budget,
            principal: self.principal.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a runbook entry in status created.
pub fn new_runbook_entry(code: &str, label: &str) -> RunbookEntry {
    RunbookEntry {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        retries: 0,
        country: "".to_string(),
        budget: 0.0,
        principal: "".to_string(),
        namespace: "".to_string(),
        status: RunbookEntryStatus::Created,
    }
}
