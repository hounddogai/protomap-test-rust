use super::*;

use acme_kit::text;

/// Where a rollback policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackPolicyStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollbackPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollbackPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPolicy {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub sequence: i64,
    pub updated_at: i64,
    pub priority: i64,
    pub phone: String,
    pub key_id: String,
    pub cost: f64,
    pub category: String,
    pub status: RollbackPolicyStatus,
}

impl RollbackPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackPolicyStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackPolicyStatus) -> bool {
        match self.status {
            RollbackPolicyStatus::Proposed => {
                match next {
                    RollbackPolicyStatus::Approved | RollbackPolicyStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackPolicyStatus::Approved => {
                match next {
                    RollbackPolicyStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackPolicyStatus::Rejected => {
                return next == RollbackPolicyStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback policy {} cannot move from {} to {}",
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
        format!("{} {}", "rollback policy", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.updated_at as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackPolicy {
        RollbackPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            sequence: self.sequence,
            updated_at: self.updated_at,
            priority: self.priority,
            phone: self.phone.clone(),
            key_id: self.key_id.clone(),
            cost: self.cost,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rollback policy in status proposed.
pub fn new_rollback_policy(code: &str, title: &str) -> RollbackPolicy {
    RollbackPolicy {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        sequence: 0,
        updated_at: 0,
        priority: 0,
        phone: "".to_string(),
        key_id: "".to_string(),
        cost: 0.0,
        category: "".to_string(),
        status: RollbackPolicyStatus::Proposed,
    }
}
