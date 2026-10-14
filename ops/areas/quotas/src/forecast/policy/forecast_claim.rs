use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a forecast claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ForecastClaimStatus {
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

impl std::fmt::Display for ForecastClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastClaim {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub fingerprint: String,
    pub updated_at: i64,
    pub pinned: bool,
    pub attributes: BTreeMap<String, String>,
    pub cluster: String,
    pub status: ForecastClaimStatus,
}

impl ForecastClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastClaimStatus) -> bool {
        match self.status {
            ForecastClaimStatus::Idle => {
                match next {
                    ForecastClaimStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastClaimStatus::Warming => {
                match next {
                    ForecastClaimStatus::Ready | ForecastClaimStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastClaimStatus::Ready => {
                return next == ForecastClaimStatus::Draining || next == ForecastClaimStatus::Stopped;
            }
            ForecastClaimStatus::Draining => {
                match next {
                    ForecastClaimStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ForecastClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_total(&self) -> f64 {
        ((self.updated_at as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastClaim {
        ForecastClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            fingerprint: self.fingerprint.clone(),
            updated_at: self.updated_at,
            pinned: self.pinned,
            attributes: self.attributes.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a forecast claim in status idle.
pub fn new_forecast_claim(code: &str, label: &str) -> ForecastClaim {
    ForecastClaim {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        fingerprint: "".to_string(),
        updated_at: 0,
        pinned: false,
        attributes: BTreeMap::new(),
        cluster: "".to_string(),
        status: ForecastClaimStatus::Idle,
    }
}
