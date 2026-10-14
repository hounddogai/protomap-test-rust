use super::*;

use acme_kit::text;

/// Where a adaptive query adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveQueryAdjustmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveQueryAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveQueryAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive query adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQueryAdjustment {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub contact_email: String,
    pub priority: i64,
    pub status: AdaptiveQueryAdjustmentStatus,
}

impl AdaptiveQueryAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveQueryAdjustmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveQueryAdjustmentStatus) -> bool {
        match self.status {
            AdaptiveQueryAdjustmentStatus::Requested => {
                return next == AdaptiveQueryAdjustmentStatus::Revoked;
            }
            AdaptiveQueryAdjustmentStatus::Granted => {
                return next == AdaptiveQueryAdjustmentStatus::Revoked;
            }
            AdaptiveQueryAdjustmentStatus::Revoked => {
                match next {
                    AdaptiveQueryAdjustmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: AdaptiveQueryAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive query adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.minimum + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveQueryAdjustment {
        AdaptiveQueryAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            contact_email: self.contact_email.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive query adjustment in status requested.
pub fn new_adaptive_query_adjustment(code: &str, title: &str) -> AdaptiveQueryAdjustment {
    AdaptiveQueryAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        contact_email: "".to_string(),
        priority: 0,
        status: AdaptiveQueryAdjustmentStatus::Requested,
    }
}
