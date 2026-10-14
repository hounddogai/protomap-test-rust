use super::*;

use acme_kit::text;

/// Where a global overage threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalOverageThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalOverageThresholdStatus {
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

impl std::fmt::Display for GlobalOverageThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global overage threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOverageThreshold {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub fingerprint: String,
    pub cluster: String,
    pub score: f64,
    pub status: GlobalOverageThresholdStatus,
}

impl GlobalOverageThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalOverageThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalOverageThresholdStatus) -> bool {
        match self.status {
            GlobalOverageThresholdStatus::Idle => {
                match next {
                    GlobalOverageThresholdStatus::Ready | GlobalOverageThresholdStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalOverageThresholdStatus::Warming => {
                return next == GlobalOverageThresholdStatus::Draining || next == GlobalOverageThresholdStatus::Stopped;
            }
            GlobalOverageThresholdStatus::Ready => {
                return next == GlobalOverageThresholdStatus::Draining;
            }
            GlobalOverageThresholdStatus::Draining => {
                match next {
                    GlobalOverageThresholdStatus::Stopped => {
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

    pub fn move_to(&mut self, next: GlobalOverageThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global overage threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> GlobalOverageThreshold {
        GlobalOverageThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            fingerprint: self.fingerprint.clone(),
            cluster: self.cluster.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global overage threshold in status idle.
pub fn new_global_overage_threshold(code: &str, title: &str) -> GlobalOverageThreshold {
    GlobalOverageThreshold {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        fingerprint: "".to_string(),
        cluster: "".to_string(),
        score: 0.0,
        status: GlobalOverageThresholdStatus::Idle,
    }
}
