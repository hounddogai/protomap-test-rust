use super::*;

use acme_kit::text;

/// Where a default suspension metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSuspensionMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultSuspensionMetricStatus {
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

impl std::fmt::Display for DefaultSuspensionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default suspension metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSuspensionMetric {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub ratio: f64,
    pub ceiling: f64,
    pub timezone: String,
    pub archived: bool,
    pub status: DefaultSuspensionMetricStatus,
}

impl DefaultSuspensionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSuspensionMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSuspensionMetricStatus) -> bool {
        match self.status {
            DefaultSuspensionMetricStatus::Idle => {
                return next == DefaultSuspensionMetricStatus::Warming;
            }
            DefaultSuspensionMetricStatus::Warming => {
                return next == DefaultSuspensionMetricStatus::Ready || next == DefaultSuspensionMetricStatus::Stopped;
            }
            DefaultSuspensionMetricStatus::Ready => {
                match next {
                    DefaultSuspensionMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSuspensionMetricStatus::Draining => {
                match next {
                    DefaultSuspensionMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultSuspensionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default suspension metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default suspension metric", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.ceiling * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultSuspensionMetric {
        DefaultSuspensionMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            ratio: self.ratio,
            ceiling: self.ceiling,
            timezone: self.timezone.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default suspension metric in status idle.
pub fn new_default_suspension_metric(code: &str, title: &str) -> DefaultSuspensionMetric {
    DefaultSuspensionMetric {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        ratio: 0.0,
        ceiling: 0.0,
        timezone: "".to_string(),
        archived: false,
        status: DefaultSuspensionMetricStatus::Idle,
    }
}
