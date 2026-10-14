use super::*;

use acme_kit::text;

/// Where a identity window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityWindowStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IdentityWindowStatus {
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

impl std::fmt::Display for IdentityWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity window as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityWindow {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub margin: f64,
    pub sequence: i64,
    pub contact_email: String,
    pub amount: f64,
    pub country: String,
    pub quantity: i64,
    pub external_ref: String,
    pub capacity: i64,
    pub status: IdentityWindowStatus,
}

impl IdentityWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityWindowStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityWindowStatus) -> bool {
        match self.status {
            IdentityWindowStatus::Planned => {
                match next {
                    IdentityWindowStatus::InProgress | IdentityWindowStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityWindowStatus::Scheduled => {
                return next == IdentityWindowStatus::Completed;
            }
            IdentityWindowStatus::InProgress => {
                return next == IdentityWindowStatus::Completed || next == IdentityWindowStatus::Cancelled;
            }
            IdentityWindowStatus::Completed => {
                match next {
                    IdentityWindowStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: IdentityWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "identity window", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.amount + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityWindow {
        IdentityWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            margin: self.margin,
            sequence: self.sequence,
            contact_email: self.contact_email.clone(),
            amount: self.amount,
            country: self.country.clone(),
            quantity: self.quantity,
            external_ref: self.external_ref.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a identity window in status planned.
pub fn new_identity_window(code: &str, label: &str) -> IdentityWindow {
    IdentityWindow {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        margin: 0.0,
        sequence: 0,
        contact_email: "".to_string(),
        amount: 0.0,
        country: "".to_string(),
        quantity: 0,
        external_ref: "".to_string(),
        capacity: 0,
        status: IdentityWindowStatus::Planned,
    }
}
