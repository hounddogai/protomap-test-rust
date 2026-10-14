use super::*;

use acme_kit::text;

/// Where a action item mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ActionItemMappingStatus {
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

impl std::fmt::Display for ActionItemMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMapping {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub contact_email: String,
    pub currency: String,
    pub ceiling: f64,
    pub attempts: i64,
    pub threshold: f64,
    pub status: ActionItemMappingStatus,
}

impl ActionItemMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemMappingStatus) -> bool {
        match self.status {
            ActionItemMappingStatus::Planned => {
                return next == ActionItemMappingStatus::Completed;
            }
            ActionItemMappingStatus::Scheduled => {
                return next == ActionItemMappingStatus::Completed || next == ActionItemMappingStatus::Cancelled;
            }
            ActionItemMappingStatus::InProgress => {
                return next == ActionItemMappingStatus::Cancelled;
            }
            ActionItemMappingStatus::Completed => {
                return next == ActionItemMappingStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.threshold * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemMapping {
        ActionItemMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            contact_email: self.contact_email.clone(),
            currency: self.currency.clone(),
            ceiling: self.ceiling,
            attempts: self.attempts,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a action item mapping in status planned.
pub fn new_action_item_mapping(code: &str, label: &str) -> ActionItemMapping {
    ActionItemMapping {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        contact_email: "".to_string(),
        currency: "".to_string(),
        ceiling: 0.0,
        attempts: 0,
        threshold: 0.0,
        status: ActionItemMappingStatus::Planned,
    }
}
