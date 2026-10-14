use super::*;

use acme_kit::text;

/// Where a federation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FederationVarianceStatus {
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

impl std::fmt::Display for FederationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationVariance {
    pub code: String,
    pub name: String,
    pub discount: f64,
    pub checksum: String,
    pub cluster: String,
    pub ceiling: f64,
    pub audience: String,
    pub grace_days: i64,
    pub burst: i64,
    pub status: FederationVarianceStatus,
}

impl FederationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationVarianceStatus) -> bool {
        match self.status {
            FederationVarianceStatus::Idle => {
                return next == FederationVarianceStatus::Warming;
            }
            FederationVarianceStatus::Warming => {
                return next == FederationVarianceStatus::Stopped;
            }
            FederationVarianceStatus::Ready => {
                match next {
                    FederationVarianceStatus::Draining | FederationVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationVarianceStatus::Draining => {
                return next == FederationVarianceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "federation variance", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ceiling - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationVariance {
        FederationVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            discount: self.discount,
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            ceiling: self.ceiling,
            audience: self.audience.clone(),
            grace_days: self.grace_days,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a federation variance in status idle.
pub fn new_federation_variance(code: &str, name: &str) -> FederationVariance {
    FederationVariance {
        code: code.to_string(),
        name: name.to_string(),
        discount: 0.0,
        checksum: "".to_string(),
        cluster: "".to_string(),
        ceiling: 0.0,
        audience: "".to_string(),
        grace_days: 0,
        burst: 0,
        status: FederationVarianceStatus::Idle,
    }
}
