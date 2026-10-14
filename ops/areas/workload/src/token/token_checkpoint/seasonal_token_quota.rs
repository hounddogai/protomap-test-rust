use super::*;

use acme_kit::text;

/// Where a seasonal token quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTokenQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SeasonalTokenQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SeasonalTokenQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal token quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenQuota {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub attempts: i64,
    pub active: bool,
    pub issuer: String,
    pub pinned: bool,
    pub status: SeasonalTokenQuotaStatus,
}

impl SeasonalTokenQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTokenQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTokenQuotaStatus) -> bool {
        match self.status {
            SeasonalTokenQuotaStatus::Created => {
                match next {
                    SeasonalTokenQuotaStatus::Published | SeasonalTokenQuotaStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTokenQuotaStatus::Validated => {
                return next == SeasonalTokenQuotaStatus::Published;
            }
            SeasonalTokenQuotaStatus::Published => {
                match next {
                    SeasonalTokenQuotaStatus::Retired => {
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

    pub fn move_to(&mut self, next: SeasonalTokenQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal token quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTokenQuota {
        SeasonalTokenQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            attempts: self.attempts,
            active: self.active,
            issuer: self.issuer.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal token quota in status created.
pub fn new_seasonal_token_quota(code: &str, name: &str) -> SeasonalTokenQuota {
    SeasonalTokenQuota {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        attempts: 0,
        active: false,
        issuer: "".to_string(),
        pinned: false,
        status: SeasonalTokenQuotaStatus::Created,
    }
}
