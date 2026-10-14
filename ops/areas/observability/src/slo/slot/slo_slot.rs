use super::*;

use acme_kit::text;

/// Where a slo slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SloSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SloSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo slot as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloSlot {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub name: String,
    pub segment: String,
    pub source: String,
    pub timezone: String,
    pub status: SloSlotStatus,
}

impl SloSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloSlotStatus) -> bool {
        match self.status {
            SloSlotStatus::Requested => {
                return next == SloSlotStatus::Revoked || next == SloSlotStatus::Expired;
            }
            SloSlotStatus::Granted => {
                return next == SloSlotStatus::Revoked || next == SloSlotStatus::Expired;
            }
            SloSlotStatus::Revoked => {
                match next {
                    SloSlotStatus::Expired => {
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

    pub fn move_to(&mut self, next: SloSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "slo slot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> SloSlot {
        SloSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            name: self.name.clone(),
            segment: self.segment.clone(),
            source: self.source.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a slo slot in status requested.
pub fn new_slo_slot(code: &str, title: &str) -> SloSlot {
    SloSlot {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        name: "".to_string(),
        segment: "".to_string(),
        source: "".to_string(),
        timezone: "".to_string(),
        status: SloSlotStatus::Requested,
    }
}
