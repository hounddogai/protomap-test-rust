use super::*;

use acme_kit::text;

/// Where a tenant milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TenantMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TenantMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tenant milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantMilestone {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub cipher: String,
    pub fingerprint: String,
    pub title: String,
    pub tenant_id: String,
    pub capacity: i64,
    pub version: i64,
    pub status: TenantMilestoneStatus,
}

impl TenantMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantMilestoneStatus) -> bool {
        match self.status {
            TenantMilestoneStatus::Requested => {
                return next == TenantMilestoneStatus::Expired;
            }
            TenantMilestoneStatus::Granted => {
                match next {
                    TenantMilestoneStatus::Revoked | TenantMilestoneStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantMilestoneStatus::Revoked => {
                return next == TenantMilestoneStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.capacity as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantMilestone {
        TenantMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            cipher: self.cipher.clone(),
            fingerprint: self.fingerprint.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            capacity: self.capacity,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tenant milestone in status requested.
pub fn new_tenant_milestone(code: &str, label: &str) -> TenantMilestone {
    TenantMilestone {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        cipher: "".to_string(),
        fingerprint: "".to_string(),
        title: "".to_string(),
        tenant_id: "".to_string(),
        capacity: 0,
        version: 0,
        status: TenantMilestoneStatus::Requested,
    }
}
