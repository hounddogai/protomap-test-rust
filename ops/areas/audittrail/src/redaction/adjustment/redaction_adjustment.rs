use super::*;

use acme_kit::text;

/// Where a redaction adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionAdjustmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RedactionAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RedactionAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAdjustment {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub quota_units: i64,
    pub floor_value: f64,
    pub tags: Vec<String>,
    pub archived: bool,
    pub phone: String,
    pub weight: f64,
    pub reviewer: String,
    pub status: RedactionAdjustmentStatus,
}

impl RedactionAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionAdjustmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionAdjustmentStatus) -> bool {
        match self.status {
            RedactionAdjustmentStatus::Requested => {
                match next {
                    RedactionAdjustmentStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionAdjustmentStatus::Granted => {
                return next == RedactionAdjustmentStatus::Revoked || next == RedactionAdjustmentStatus::Expired;
            }
            RedactionAdjustmentStatus::Revoked => {
                return next == RedactionAdjustmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "redaction adjustment", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.quota_units as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionAdjustment {
        RedactionAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            quota_units: self.quota_units,
            floor_value: self.floor_value,
            tags: self.tags.clone(),
            archived: self.archived,
            phone: self.phone.clone(),
            weight: self.weight,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a redaction adjustment in status requested.
pub fn new_redaction_adjustment(code: &str, label: &str) -> RedactionAdjustment {
    RedactionAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        quota_units: 0,
        floor_value: 0.0,
        tags: Vec::new(),
        archived: false,
        phone: "".to_string(),
        weight: 0.0,
        reviewer: "".to_string(),
        status: RedactionAdjustmentStatus::Requested,
    }
}
