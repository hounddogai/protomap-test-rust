use super::*;

use acme_kit::text;

/// Where a default release threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultReleaseThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultReleaseThresholdStatus {
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

impl std::fmt::Display for DefaultReleaseThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default release threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseThreshold {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub ends_at: i64,
    pub sequence: i64,
    pub attempts: i64,
    pub tenant_id: String,
    pub version: i64,
    pub segment: String,
    pub status: DefaultReleaseThresholdStatus,
}

impl DefaultReleaseThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultReleaseThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultReleaseThresholdStatus) -> bool {
        match self.status {
            DefaultReleaseThresholdStatus::Idle => {
                return next == DefaultReleaseThresholdStatus::Warming;
            }
            DefaultReleaseThresholdStatus::Warming => {
                return next == DefaultReleaseThresholdStatus::Ready || next == DefaultReleaseThresholdStatus::Draining;
            }
            DefaultReleaseThresholdStatus::Ready => {
                return next == DefaultReleaseThresholdStatus::Draining
                    || next == DefaultReleaseThresholdStatus::Stopped;
            }
            DefaultReleaseThresholdStatus::Draining => {
                match next {
                    DefaultReleaseThresholdStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultReleaseThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default release threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.version as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultReleaseThreshold {
        DefaultReleaseThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            ends_at: self.ends_at,
            sequence: self.sequence,
            attempts: self.attempts,
            tenant_id: self.tenant_id.clone(),
            version: self.version,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a default release threshold in status idle.
pub fn new_default_release_threshold(code: &str, name: &str) -> DefaultReleaseThreshold {
    DefaultReleaseThreshold {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        ends_at: 0,
        sequence: 0,
        attempts: 0,
        tenant_id: "".to_string(),
        version: 0,
        segment: "".to_string(),
        status: DefaultReleaseThresholdStatus::Idle,
    }
}
