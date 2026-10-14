use super::*;

use acme_kit::text;

/// Where a adaptive alert quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAlertQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveAlertQuotaStatus {
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

impl std::fmt::Display for AdaptiveAlertQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive alert quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertQuota {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub host: String,
    pub score: f64,
    pub locale: String,
    pub title: String,
    pub margin: f64,
    pub rate: f64,
    pub status: AdaptiveAlertQuotaStatus,
}

impl AdaptiveAlertQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAlertQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAlertQuotaStatus) -> bool {
        match self.status {
            AdaptiveAlertQuotaStatus::Idle => {
                return next == AdaptiveAlertQuotaStatus::Ready;
            }
            AdaptiveAlertQuotaStatus::Warming => {
                return next == AdaptiveAlertQuotaStatus::Draining;
            }
            AdaptiveAlertQuotaStatus::Ready => {
                return next == AdaptiveAlertQuotaStatus::Draining || next == AdaptiveAlertQuotaStatus::Stopped;
            }
            AdaptiveAlertQuotaStatus::Draining => {
                return next == AdaptiveAlertQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveAlertQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive alert quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.score - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAlertQuota {
        AdaptiveAlertQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            host: self.host.clone(),
            score: self.score,
            locale: self.locale.clone(),
            title: self.title.clone(),
            margin: self.margin,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive alert quota in status idle.
pub fn new_adaptive_alert_quota(code: &str, label: &str) -> AdaptiveAlertQuota {
    AdaptiveAlertQuota {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        host: "".to_string(),
        score: 0.0,
        locale: "".to_string(),
        title: "".to_string(),
        margin: 0.0,
        rate: 0.0,
        status: AdaptiveAlertQuotaStatus::Idle,
    }
}
