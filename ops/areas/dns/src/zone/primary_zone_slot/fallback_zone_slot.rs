use super::*;

use acme_kit::text;

/// Where a fallback zone slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackZoneSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackZoneSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for FallbackZoneSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback zone slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneSlot {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub source: String,
    pub margin: f64,
    pub updated_at: i64,
    pub locale: String,
    pub audience: String,
    pub weight: f64,
    pub status: FallbackZoneSlotStatus,
}

impl FallbackZoneSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackZoneSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackZoneSlotStatus) -> bool {
        match self.status {
            FallbackZoneSlotStatus::Open => {
                match next {
                    FallbackZoneSlotStatus::Resolved | FallbackZoneSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackZoneSlotStatus::InReview => {
                return next == FallbackZoneSlotStatus::Resolved;
            }
            FallbackZoneSlotStatus::Resolved => {
                match next {
                    FallbackZoneSlotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackZoneSlotStatus::Reopened => {
                match next {
                    FallbackZoneSlotStatus::Archived => {
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

    pub fn move_to(&mut self, next: FallbackZoneSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback zone slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.margin + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackZoneSlot {
        FallbackZoneSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            source: self.source.clone(),
            margin: self.margin,
            updated_at: self.updated_at,
            locale: self.locale.clone(),
            audience: self.audience.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a fallback zone slot in status open.
pub fn new_fallback_zone_slot(code: &str, label: &str) -> FallbackZoneSlot {
    FallbackZoneSlot {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        source: "".to_string(),
        margin: 0.0,
        updated_at: 0,
        locale: "".to_string(),
        audience: "".to_string(),
        weight: 0.0,
        status: FallbackZoneSlotStatus::Open,
    }
}
