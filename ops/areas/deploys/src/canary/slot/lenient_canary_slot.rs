use super::*;

use acme_kit::text;

/// Where a lenient canary slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientCanarySlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientCanarySlotStatus {
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

impl std::fmt::Display for LenientCanarySlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient canary slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanarySlot {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub timezone: String,
    pub window_days: i64,
    pub maximum: f64,
    pub expires_at: i64,
    pub status: LenientCanarySlotStatus,
}

impl LenientCanarySlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientCanarySlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientCanarySlotStatus) -> bool {
        match self.status {
            LenientCanarySlotStatus::Planned => {
                match next {
                    LenientCanarySlotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientCanarySlotStatus::Scheduled => {
                return next == LenientCanarySlotStatus::InProgress;
            }
            LenientCanarySlotStatus::InProgress => {
                match next {
                    LenientCanarySlotStatus::Completed | LenientCanarySlotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientCanarySlotStatus::Completed => {
                return next == LenientCanarySlotStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientCanarySlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient canary slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.maximum - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientCanarySlot {
        LenientCanarySlot {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            timezone: self.timezone.clone(),
            window_days: self.window_days,
            maximum: self.maximum,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient canary slot in status planned.
pub fn new_lenient_canary_slot(code: &str, label: &str) -> LenientCanarySlot {
    LenientCanarySlot {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        timezone: "".to_string(),
        window_days: 0,
        maximum: 0.0,
        expires_at: 0,
        status: LenientCanarySlotStatus::Planned,
    }
}
