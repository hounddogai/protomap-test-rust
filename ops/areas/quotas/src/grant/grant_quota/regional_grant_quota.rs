use super::*;

use acme_kit::text;

/// Where a regional grant quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalGrantQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalGrantQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RegionalGrantQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional grant quota as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantQuota {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub port: i64,
    pub attempts: i64,
    pub weight: f64,
    pub budget: f64,
    pub starts_at: i64,
    pub status: RegionalGrantQuotaStatus,
}

impl RegionalGrantQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalGrantQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalGrantQuotaStatus) -> bool {
        match self.status {
            RegionalGrantQuotaStatus::Idle => {
                return next == RegionalGrantQuotaStatus::Warming;
            }
            RegionalGrantQuotaStatus::Warming => {
                return next == RegionalGrantQuotaStatus::Stopped;
            }
            RegionalGrantQuotaStatus::Ready => {
                match next {
                    RegionalGrantQuotaStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalGrantQuotaStatus::Draining => {
                return next == RegionalGrantQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalGrantQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional grant quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.weight * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalGrantQuota {
        RegionalGrantQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            port: self.port,
            attempts: self.attempts,
            weight: self.weight,
            budget: self.budget,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional grant quota in status idle.
pub fn new_regional_grant_quota(code: &str, name: &str) -> RegionalGrantQuota {
    RegionalGrantQuota {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        port: 0,
        attempts: 0,
        weight: 0.0,
        budget: 0.0,
        starts_at: 0,
        status: RegionalGrantQuotaStatus::Idle,
    }
}
