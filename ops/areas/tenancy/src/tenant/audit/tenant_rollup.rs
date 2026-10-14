use super::*;

use acme_kit::text;

/// Where a tenant rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TenantRollupStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TenantRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TenantRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tenant rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantRollup {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub fingerprint: String,
    pub maximum: f64,
    pub principal: String,
    pub cost: f64,
    pub scope: String,
    pub status: TenantRollupStatus,
}

impl TenantRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TenantRollupStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TenantRollupStatus) -> bool {
        match self.status {
            TenantRollupStatus::Created => {
                match next {
                    TenantRollupStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TenantRollupStatus::Validated => {
                return next == TenantRollupStatus::Published || next == TenantRollupStatus::Retired;
            }
            TenantRollupStatus::Published => {
                return next == TenantRollupStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TenantRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tenant rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.cost - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TenantRollup {
        TenantRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            fingerprint: self.fingerprint.clone(),
            maximum: self.maximum,
            principal: self.principal.clone(),
            cost: self.cost,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tenant rollup in status created.
pub fn new_tenant_rollup(code: &str, title: &str) -> TenantRollup {
    TenantRollup {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        fingerprint: "".to_string(),
        maximum: 0.0,
        principal: "".to_string(),
        cost: 0.0,
        scope: "".to_string(),
        status: TenantRollupStatus::Created,
    }
}
