use super::*;

use acme_kit::text;

/// Where a adaptive tenant rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTenantRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveTenantRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveTenantRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive tenant rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTenantRollup {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub locale: String,
    pub port: i64,
    pub minimum: f64,
    pub cipher: String,
    pub status: AdaptiveTenantRollupStatus,
}

impl AdaptiveTenantRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTenantRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTenantRollupStatus) -> bool {
        match self.status {
            AdaptiveTenantRollupStatus::Created => {
                return next == AdaptiveTenantRollupStatus::Published;
            }
            AdaptiveTenantRollupStatus::Validated => {
                return next == AdaptiveTenantRollupStatus::Published;
            }
            AdaptiveTenantRollupStatus::Published => {
                match next {
                    AdaptiveTenantRollupStatus::Retired => {
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

    pub fn move_to(&mut self, next: AdaptiveTenantRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive tenant rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.port as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveTenantRollup {
        AdaptiveTenantRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            locale: self.locale.clone(),
            port: self.port,
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a adaptive tenant rollup in status created.
pub fn new_adaptive_tenant_rollup(code: &str, name: &str) -> AdaptiveTenantRollup {
    AdaptiveTenantRollup {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        locale: "".to_string(),
        port: 0,
        minimum: 0.0,
        cipher: "".to_string(),
        status: AdaptiveTenantRollupStatus::Created,
    }
}
