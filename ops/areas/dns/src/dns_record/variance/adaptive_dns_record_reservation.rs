use super::*;

use acme_kit::text;

/// Where a adaptive dns record reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveDnsRecordReservationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveDnsRecordReservationStatus {
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

impl std::fmt::Display for AdaptiveDnsRecordReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive dns record reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDnsRecordReservation {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub amount: f64,
    pub audience: String,
    pub status: AdaptiveDnsRecordReservationStatus,
}

impl AdaptiveDnsRecordReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveDnsRecordReservationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveDnsRecordReservationStatus) -> bool {
        match self.status {
            AdaptiveDnsRecordReservationStatus::Planned => {
                match next {
                    AdaptiveDnsRecordReservationStatus::Scheduled | AdaptiveDnsRecordReservationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDnsRecordReservationStatus::Scheduled => {
                match next {
                    AdaptiveDnsRecordReservationStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDnsRecordReservationStatus::InProgress => {
                return next == AdaptiveDnsRecordReservationStatus::Cancelled;
            }
            AdaptiveDnsRecordReservationStatus::Completed => {
                return next == AdaptiveDnsRecordReservationStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveDnsRecordReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive dns record reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> AdaptiveDnsRecordReservation {
        AdaptiveDnsRecordReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            amount: self.amount,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive dns record reservation in status planned.
pub fn new_adaptive_dns_record_reservation(code: &str, name: &str) -> AdaptiveDnsRecordReservation {
    AdaptiveDnsRecordReservation {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        amount: 0.0,
        audience: "".to_string(),
        status: AdaptiveDnsRecordReservationStatus::Planned,
    }
}
