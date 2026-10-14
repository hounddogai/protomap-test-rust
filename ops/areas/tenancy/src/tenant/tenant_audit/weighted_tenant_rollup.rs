use super::*;

use acme_kit::text;

/// Where a weighted tenant rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTenantRollupStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedTenantRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedTenantRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted tenant rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTenantRollup {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub retries: i64,
    pub revision: i64,
    pub cost: f64,
    pub expires_at: i64,
    pub country: String,
    pub status: WeightedTenantRollupStatus,
}

impl WeightedTenantRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTenantRollupStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTenantRollupStatus) -> bool {
        match self.status {
            WeightedTenantRollupStatus::Requested => {
                return next == WeightedTenantRollupStatus::Granted;
            }
            WeightedTenantRollupStatus::Granted => {
                return next == WeightedTenantRollupStatus::Revoked;
            }
            WeightedTenantRollupStatus::Revoked => {
                match next {
                    WeightedTenantRollupStatus::Expired => {
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

    pub fn move_to(&mut self, next: WeightedTenantRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted tenant rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted tenant rollup", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.expires_at as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTenantRollup {
        WeightedTenantRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            retries: self.retries,
            revision: self.revision,
            cost: self.cost,
            expires_at: self.expires_at,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted tenant rollup in status requested.
pub fn new_weighted_tenant_rollup(code: &str, title: &str) -> WeightedTenantRollup {
    WeightedTenantRollup {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        retries: 0,
        revision: 0,
        cost: 0.0,
        expires_at: 0,
        country: "".to_string(),
        status: WeightedTenantRollupStatus::Requested,
    }
}
