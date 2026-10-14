use super::*;

use acme_kit::text;

/// Where a sink variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SinkVarianceStatus {
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

impl std::fmt::Display for SinkVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a sink variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkVariance {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub tags: Vec<String>,
    pub limit: i64,
    pub cluster: String,
    pub scope: String,
    pub status: SinkVarianceStatus,
}

impl SinkVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkVarianceStatus) -> bool {
        match self.status {
            SinkVarianceStatus::Idle => {
                match next {
                    SinkVarianceStatus::Draining | SinkVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkVarianceStatus::Warming => {
                match next {
                    SinkVarianceStatus::Ready | SinkVarianceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkVarianceStatus::Ready => {
                match next {
                    SinkVarianceStatus::Draining | SinkVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkVarianceStatus::Draining => {
                return next == SinkVarianceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "sink variance", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> SinkVariance {
        SinkVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            tags: self.tags.clone(),
            limit: self.limit,
            cluster: self.cluster.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a sink variance in status idle.
pub fn new_sink_variance(code: &str, title: &str) -> SinkVariance {
    SinkVariance {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        tags: Vec::new(),
        limit: 0,
        cluster: "".to_string(),
        scope: "".to_string(),
        status: SinkVarianceStatus::Idle,
    }
}
