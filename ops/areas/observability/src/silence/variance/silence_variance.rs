use super::*;

use acme_kit::text;

/// Where a silence variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SilenceVarianceStatus {
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

impl std::fmt::Display for SilenceVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One silence variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceVariance {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub attempts: i64,
    pub cluster: String,
    pub title: String,
    pub score: f64,
    pub status: SilenceVarianceStatus,
}

impl SilenceVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceVarianceStatus) -> bool {
        match self.status {
            SilenceVarianceStatus::Idle => {
                return next == SilenceVarianceStatus::Draining;
            }
            SilenceVarianceStatus::Warming => {
                match next {
                    SilenceVarianceStatus::Draining | SilenceVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceVarianceStatus::Ready => {
                match next {
                    SilenceVarianceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceVarianceStatus::Draining => {
                return next == SilenceVarianceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "silence variance", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.score + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceVariance {
        SilenceVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            attempts: self.attempts,
            cluster: self.cluster.clone(),
            title: self.title.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a silence variance in status idle.
pub fn new_silence_variance(code: &str, label: &str) -> SilenceVariance {
    SilenceVariance {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        attempts: 0,
        cluster: "".to_string(),
        title: "".to_string(),
        score: 0.0,
        status: SilenceVarianceStatus::Idle,
    }
}
