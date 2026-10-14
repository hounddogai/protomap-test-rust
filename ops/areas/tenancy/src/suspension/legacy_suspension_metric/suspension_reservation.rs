use super::*;

use acme_kit::text;

/// Where a suspension reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SuspensionReservationStatus {
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

impl std::fmt::Display for SuspensionReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionReservation {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub title: String,
    pub phone: String,
    pub status: SuspensionReservationStatus,
}

impl SuspensionReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionReservationStatus) -> bool {
        match self.status {
            SuspensionReservationStatus::Draft => {
                return next == SuspensionReservationStatus::Suspended;
            }
            SuspensionReservationStatus::Pending => {
                match next {
                    SuspensionReservationStatus::Suspended | SuspensionReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionReservationStatus::Active => {
                return next == SuspensionReservationStatus::Closed;
            }
            SuspensionReservationStatus::Suspended => {
                return next == SuspensionReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension reservation {} cannot move from {} to {}",
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
        self.attempts as f64
    }

    pub fn copy(&self) -> SuspensionReservation {
        SuspensionReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            title: self.title.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a suspension reservation in status draft.
pub fn new_suspension_reservation(code: &str, label: &str) -> SuspensionReservation {
    SuspensionReservation {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        title: "".to_string(),
        phone: "".to_string(),
        status: SuspensionReservationStatus::Draft,
    }
}
