use super::*;

use acme_kit::text;

/// Where a fallback plan digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPlanDigestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackPlanDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackPlanDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback plan digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanDigest {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub category: String,
    pub limit: i64,
    pub created_at: i64,
    pub description: String,
    pub discount: f64,
    pub quota_units: i64,
    pub grace_days: i64,
    pub country: String,
    pub status: FallbackPlanDigestStatus,
}

impl FallbackPlanDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPlanDigestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPlanDigestStatus) -> bool {
        match self.status {
            FallbackPlanDigestStatus::Created => {
                return next == FallbackPlanDigestStatus::Validated || next == FallbackPlanDigestStatus::Published;
            }
            FallbackPlanDigestStatus::Validated => {
                match next {
                    FallbackPlanDigestStatus::Published | FallbackPlanDigestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPlanDigestStatus::Published => {
                match next {
                    FallbackPlanDigestStatus::Retired => {
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

    pub fn move_to(&mut self, next: FallbackPlanDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback plan digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.created_at as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPlanDigest {
        FallbackPlanDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            category: self.category.clone(),
            limit: self.limit,
            created_at: self.created_at,
            description: self.description.clone(),
            discount: self.discount,
            quota_units: self.quota_units,
            grace_days: self.grace_days,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a fallback plan digest in status created.
pub fn new_fallback_plan_digest(code: &str, name: &str) -> FallbackPlanDigest {
    FallbackPlanDigest {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        category: "".to_string(),
        limit: 0,
        created_at: 0,
        description: "".to_string(),
        discount: 0.0,
        quota_units: 0,
        grace_days: 0,
        country: "".to_string(),
        status: FallbackPlanDigestStatus::Created,
    }
}
