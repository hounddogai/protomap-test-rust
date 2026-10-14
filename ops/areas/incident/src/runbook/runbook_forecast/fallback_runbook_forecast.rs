use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback runbook forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRunbookForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackRunbookForecastStatus {
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

impl std::fmt::Display for FallbackRunbookForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback runbook forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookForecast {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub reviewer: String,
    pub revision: i64,
    pub namespace: String,
    pub channel: String,
    pub tenant_id: String,
    pub margin: f64,
    pub created_at: i64,
    pub status: FallbackRunbookForecastStatus,
}

impl FallbackRunbookForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRunbookForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRunbookForecastStatus) -> bool {
        match self.status {
            FallbackRunbookForecastStatus::Idle => {
                return next == FallbackRunbookForecastStatus::Warming || next == FallbackRunbookForecastStatus::Ready;
            }
            FallbackRunbookForecastStatus::Warming => {
                match next {
                    FallbackRunbookForecastStatus::Ready | FallbackRunbookForecastStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRunbookForecastStatus::Ready => {
                return next == FallbackRunbookForecastStatus::Stopped;
            }
            FallbackRunbookForecastStatus::Draining => {
                return next == FallbackRunbookForecastStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackRunbookForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback runbook forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn net_exposure(&self) -> f64 {
        ((self.created_at as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRunbookForecast {
        FallbackRunbookForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            attributes: self.attributes.clone(),
            reviewer: self.reviewer.clone(),
            revision: self.revision,
            namespace: self.namespace.clone(),
            channel: self.channel.clone(),
            tenant_id: self.tenant_id.clone(),
            margin: self.margin,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback runbook forecast in status idle.
pub fn new_fallback_runbook_forecast(code: &str, title: &str) -> FallbackRunbookForecast {
    FallbackRunbookForecast {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        attributes: BTreeMap::new(),
        reviewer: "".to_string(),
        revision: 0,
        namespace: "".to_string(),
        channel: "".to_string(),
        tenant_id: "".to_string(),
        margin: 0.0,
        created_at: 0,
        status: FallbackRunbookForecastStatus::Idle,
    }
}
