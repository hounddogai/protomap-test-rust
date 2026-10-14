use super::*;

use acme_kit::text;

/// Where a slo forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SloForecastStatus {
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

impl std::fmt::Display for SloForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A slo forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SloForecast {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub quantity: i64,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub issuer: String,
    pub checksum: String,
    pub batch_size: i64,
    pub status: SloForecastStatus,
}

impl SloForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloForecastStatus) -> bool {
        match self.status {
            SloForecastStatus::Idle => {
                match next {
                    SloForecastStatus::Warming | SloForecastStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloForecastStatus::Warming => {
                return next == SloForecastStatus::Stopped;
            }
            SloForecastStatus::Ready => {
                return next == SloForecastStatus::Draining;
            }
            SloForecastStatus::Draining => {
                return next == SloForecastStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.expires_at as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloForecast {
        SloForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            quantity: self.quantity,
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            issuer: self.issuer.clone(),
            checksum: self.checksum.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a slo forecast in status idle.
pub fn new_slo_forecast(code: &str, title: &str) -> SloForecast {
    SloForecast {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        quantity: 0,
        expires_at: 0,
        tags: Vec::new(),
        issuer: "".to_string(),
        checksum: "".to_string(),
        batch_size: 0,
        status: SloForecastStatus::Idle,
    }
}
