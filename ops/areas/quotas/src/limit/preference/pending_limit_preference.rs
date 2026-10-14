use super::*;

use acme_kit::text;

/// Where a pending limit preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingLimitPreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingLimitPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingLimitPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending limit preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPreference {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub rotation_days: i64,
    pub weight: f64,
    pub quantity: i64,
    pub score: f64,
    pub window_days: i64,
    pub checksum: String,
    pub status: PendingLimitPreferenceStatus,
}

impl PendingLimitPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingLimitPreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingLimitPreferenceStatus) -> bool {
        match self.status {
            PendingLimitPreferenceStatus::Created => {
                match next {
                    PendingLimitPreferenceStatus::Published | PendingLimitPreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingLimitPreferenceStatus::Validated => {
                match next {
                    PendingLimitPreferenceStatus::Published | PendingLimitPreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingLimitPreferenceStatus::Published => {
                match next {
                    PendingLimitPreferenceStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingLimitPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending limit preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.window_days as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingLimitPreference {
        PendingLimitPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            rotation_days: self.rotation_days,
            weight: self.weight,
            quantity: self.quantity,
            score: self.score,
            window_days: self.window_days,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending limit preference in status created.
pub fn new_pending_limit_preference(code: &str, label: &str) -> PendingLimitPreference {
    PendingLimitPreference {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        rotation_days: 0,
        weight: 0.0,
        quantity: 0,
        score: 0.0,
        window_days: 0,
        checksum: "".to_string(),
        status: PendingLimitPreferenceStatus::Created,
    }
}
