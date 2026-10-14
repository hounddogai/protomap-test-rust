use super::*;

use acme_kit::text;

/// Where a lenient region pin tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientRegionPinTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientRegionPinTierStatus {
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

impl std::fmt::Display for LenientRegionPinTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient region pin tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientRegionPinTier {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub quota_units: i64,
    pub active: bool,
    pub cipher: String,
    pub external_ref: String,
    pub grace_days: i64,
    pub visible: bool,
    pub namespace: String,
    pub rotation_days: i64,
    pub status: LenientRegionPinTierStatus,
}

impl LenientRegionPinTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientRegionPinTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientRegionPinTierStatus) -> bool {
        match self.status {
            LenientRegionPinTierStatus::Planned => {
                match next {
                    LenientRegionPinTierStatus::Scheduled | LenientRegionPinTierStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRegionPinTierStatus::Scheduled => {
                match next {
                    LenientRegionPinTierStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientRegionPinTierStatus::InProgress => {
                return next == LenientRegionPinTierStatus::Completed;
            }
            LenientRegionPinTierStatus::Completed => {
                match next {
                    LenientRegionPinTierStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientRegionPinTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient region pin tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.grace_days as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientRegionPinTier {
        LenientRegionPinTier {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            quota_units: self.quota_units,
            active: self.active,
            cipher: self.cipher.clone(),
            external_ref: self.external_ref.clone(),
            grace_days: self.grace_days,
            visible: self.visible,
            namespace: self.namespace.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a lenient region pin tier in status planned.
pub fn new_lenient_region_pin_tier(code: &str, label: &str) -> LenientRegionPinTier {
    LenientRegionPinTier {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        quota_units: 0,
        active: false,
        cipher: "".to_string(),
        external_ref: "".to_string(),
        grace_days: 0,
        visible: false,
        namespace: "".to_string(),
        rotation_days: 0,
        status: LenientRegionPinTierStatus::Planned,
    }
}
