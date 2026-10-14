use super::*;

use acme_kit::text;

/// Where a policy adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PolicyAdjustmentStatus {
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

impl std::fmt::Display for PolicyAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAdjustment {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub updated_at: i64,
    pub pinned: bool,
    pub audience: String,
    pub fingerprint: String,
    pub duration_minutes: i64,
    pub category: String,
    pub status: PolicyAdjustmentStatus,
}

impl PolicyAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyAdjustmentStatus) -> bool {
        match self.status {
            PolicyAdjustmentStatus::Open => {
                return next == PolicyAdjustmentStatus::Archived;
            }
            PolicyAdjustmentStatus::InReview => {
                return next == PolicyAdjustmentStatus::Reopened;
            }
            PolicyAdjustmentStatus::Resolved => {
                match next {
                    PolicyAdjustmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyAdjustmentStatus::Reopened => {
                match next {
                    PolicyAdjustmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: PolicyAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy adjustment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "policy adjustment", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyAdjustment {
        PolicyAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            updated_at: self.updated_at,
            pinned: self.pinned,
            audience: self.audience.clone(),
            fingerprint: self.fingerprint.clone(),
            duration_minutes: self.duration_minutes,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a policy adjustment in status open.
pub fn new_policy_adjustment(code: &str, name: &str) -> PolicyAdjustment {
    PolicyAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        updated_at: 0,
        pinned: false,
        audience: "".to_string(),
        fingerprint: "".to_string(),
        duration_minutes: 0,
        category: "".to_string(),
        status: PolicyAdjustmentStatus::Open,
    }
}
