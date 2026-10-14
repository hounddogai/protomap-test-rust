use super::*;

use acme_kit::text;

/// Where a capped redaction bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRedactionBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedRedactionBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedRedactionBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped redaction bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionBundle {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub grace_days: i64,
    pub priority: i64,
    pub capacity: i64,
    pub ceiling: f64,
    pub quantity: i64,
    pub status: CappedRedactionBundleStatus,
}

impl CappedRedactionBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRedactionBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRedactionBundleStatus) -> bool {
        match self.status {
            CappedRedactionBundleStatus::Proposed => {
                return next == CappedRedactionBundleStatus::Withdrawn;
            }
            CappedRedactionBundleStatus::Approved => {
                match next {
                    CappedRedactionBundleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRedactionBundleStatus::Rejected => {
                match next {
                    CappedRedactionBundleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CappedRedactionBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped redaction bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.quota_units as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRedactionBundle {
        CappedRedactionBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            grace_days: self.grace_days,
            priority: self.priority,
            capacity: self.capacity,
            ceiling: self.ceiling,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped redaction bundle in status proposed.
pub fn new_capped_redaction_bundle(code: &str, label: &str) -> CappedRedactionBundle {
    CappedRedactionBundle {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        grace_days: 0,
        priority: 0,
        capacity: 0,
        ceiling: 0.0,
        quantity: 0,
        status: CappedRedactionBundleStatus::Proposed,
    }
}
