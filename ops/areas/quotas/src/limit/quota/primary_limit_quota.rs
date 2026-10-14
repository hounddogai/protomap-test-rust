use super::*;

use acme_kit::text;

/// Where a primary limit quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryLimitQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryLimitQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryLimitQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary limit quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitQuota {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub currency: String,
    pub category: String,
    pub principal: String,
    pub archived: bool,
    pub discount: f64,
    pub priority: i64,
    pub status: PrimaryLimitQuotaStatus,
}

impl PrimaryLimitQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryLimitQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryLimitQuotaStatus) -> bool {
        match self.status {
            PrimaryLimitQuotaStatus::Created => {
                return next == PrimaryLimitQuotaStatus::Published || next == PrimaryLimitQuotaStatus::Retired;
            }
            PrimaryLimitQuotaStatus::Validated => {
                return next == PrimaryLimitQuotaStatus::Retired;
            }
            PrimaryLimitQuotaStatus::Published => {
                match next {
                    PrimaryLimitQuotaStatus::Retired => {
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

    pub fn move_to(&mut self, next: PrimaryLimitQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary limit quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.revision as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryLimitQuota {
        PrimaryLimitQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            currency: self.currency.clone(),
            category: self.category.clone(),
            principal: self.principal.clone(),
            archived: self.archived,
            discount: self.discount,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a primary limit quota in status created.
pub fn new_primary_limit_quota(code: &str, title: &str) -> PrimaryLimitQuota {
    PrimaryLimitQuota {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        currency: "".to_string(),
        category: "".to_string(),
        principal: "".to_string(),
        archived: false,
        discount: 0.0,
        priority: 0,
        status: PrimaryLimitQuotaStatus::Created,
    }
}
