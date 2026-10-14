use super::*;

use acme_kit::text;

/// Where a strict segment window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictSegmentWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictSegmentWindowStatus {
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

impl std::fmt::Display for StrictSegmentWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict segment window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentWindow {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub attempts: i64,
    pub version: i64,
    pub starts_at: i64,
    pub audience: String,
    pub threshold: f64,
    pub status: StrictSegmentWindowStatus,
}

impl StrictSegmentWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictSegmentWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictSegmentWindowStatus) -> bool {
        match self.status {
            StrictSegmentWindowStatus::Idle => {
                match next {
                    StrictSegmentWindowStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSegmentWindowStatus::Warming => {
                match next {
                    StrictSegmentWindowStatus::Ready | StrictSegmentWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSegmentWindowStatus::Ready => {
                return next == StrictSegmentWindowStatus::Stopped;
            }
            StrictSegmentWindowStatus::Draining => {
                return next == StrictSegmentWindowStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictSegmentWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict segment window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.version as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictSegmentWindow {
        StrictSegmentWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            attempts: self.attempts,
            version: self.version,
            starts_at: self.starts_at,
            audience: self.audience.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict segment window in status idle.
pub fn new_strict_segment_window(code: &str, name: &str) -> StrictSegmentWindow {
    StrictSegmentWindow {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        attempts: 0,
        version: 0,
        starts_at: 0,
        audience: "".to_string(),
        threshold: 0.0,
        status: StrictSegmentWindowStatus::Idle,
    }
}
