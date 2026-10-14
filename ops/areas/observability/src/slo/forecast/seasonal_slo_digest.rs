use super::*;

use acme_kit::text;

/// Where a seasonal slo digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalSloDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalSloDigestStatus {
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

impl std::fmt::Display for SeasonalSloDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal slo digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalSloDigest {
    pub code: String,
    pub title: String,
    pub country: String,
    pub quota_units: i64,
    pub fingerprint: String,
    pub category: String,
    pub created_at: i64,
    pub status: SeasonalSloDigestStatus,
}

impl SeasonalSloDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalSloDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalSloDigestStatus) -> bool {
        match self.status {
            SeasonalSloDigestStatus::Idle => {
                return next == SeasonalSloDigestStatus::Ready;
            }
            SeasonalSloDigestStatus::Warming => {
                match next {
                    SeasonalSloDigestStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalSloDigestStatus::Ready => {
                match next {
                    SeasonalSloDigestStatus::Draining | SeasonalSloDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalSloDigestStatus::Draining => {
                match next {
                    SeasonalSloDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalSloDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal slo digest {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.created_at as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalSloDigest {
        SeasonalSloDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            quota_units: self.quota_units,
            fingerprint: self.fingerprint.clone(),
            category: self.category.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal slo digest in status idle.
pub fn new_seasonal_slo_digest(code: &str, title: &str) -> SeasonalSloDigest {
    SeasonalSloDigest {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        quota_units: 0,
        fingerprint: "".to_string(),
        category: "".to_string(),
        created_at: 0,
        status: SeasonalSloDigestStatus::Idle,
    }
}
