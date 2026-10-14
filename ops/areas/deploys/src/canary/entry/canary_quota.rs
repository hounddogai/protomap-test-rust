use super::*;

use acme_kit::text;

/// Where a canary quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CanaryQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CanaryQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryQuota {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub locale: String,
    pub quota_units: i64,
    pub cipher: String,
    pub capacity: i64,
    pub currency: String,
    pub visible: bool,
    pub budget: f64,
    pub phone: String,
    pub status: CanaryQuotaStatus,
}

impl CanaryQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryQuotaStatus) -> bool {
        match self.status {
            CanaryQuotaStatus::Proposed => {
                match next {
                    CanaryQuotaStatus::Approved | CanaryQuotaStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryQuotaStatus::Approved => {
                match next {
                    CanaryQuotaStatus::Rejected | CanaryQuotaStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryQuotaStatus::Rejected => {
                return next == CanaryQuotaStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.quota_units as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryQuota {
        CanaryQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            locale: self.locale.clone(),
            quota_units: self.quota_units,
            cipher: self.cipher.clone(),
            capacity: self.capacity,
            currency: self.currency.clone(),
            visible: self.visible,
            budget: self.budget,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a canary quota in status proposed.
pub fn new_canary_quota(code: &str, name: &str) -> CanaryQuota {
    CanaryQuota {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        locale: "".to_string(),
        quota_units: 0,
        cipher: "".to_string(),
        capacity: 0,
        currency: "".to_string(),
        visible: false,
        budget: 0.0,
        phone: "".to_string(),
        status: CanaryQuotaStatus::Proposed,
    }
}
