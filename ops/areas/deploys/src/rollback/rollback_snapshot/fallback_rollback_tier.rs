use super::*;

use acme_kit::text;

/// Where a fallback rollback tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRollbackTierStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackRollbackTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackRollbackTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback rollback tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackTier {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub ratio: f64,
    pub limit: i64,
    pub quota_units: i64,
    pub description: String,
    pub weight: f64,
    pub duration_minutes: i64,
    pub segment: String,
    pub amount: f64,
    pub status: FallbackRollbackTierStatus,
}

impl FallbackRollbackTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRollbackTierStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRollbackTierStatus) -> bool {
        match self.status {
            FallbackRollbackTierStatus::Created => {
                match next {
                    FallbackRollbackTierStatus::Validated | FallbackRollbackTierStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRollbackTierStatus::Validated => {
                match next {
                    FallbackRollbackTierStatus::Published | FallbackRollbackTierStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRollbackTierStatus::Published => {
                return next == FallbackRollbackTierStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackRollbackTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback rollback tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback rollback tier", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRollbackTier {
        FallbackRollbackTier {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            ratio: self.ratio,
            limit: self.limit,
            quota_units: self.quota_units,
            description: self.description.clone(),
            weight: self.weight,
            duration_minutes: self.duration_minutes,
            segment: self.segment.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a fallback rollback tier in status created.
pub fn new_fallback_rollback_tier(code: &str, title: &str) -> FallbackRollbackTier {
    FallbackRollbackTier {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        ratio: 0.0,
        limit: 0,
        quota_units: 0,
        description: "".to_string(),
        weight: 0.0,
        duration_minutes: 0,
        segment: "".to_string(),
        amount: 0.0,
        status: FallbackRollbackTierStatus::Created,
    }
}
