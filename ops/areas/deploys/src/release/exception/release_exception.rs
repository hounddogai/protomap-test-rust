use super::*;

use acme_kit::text;

/// Where a release exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReleaseExceptionStatus {
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

impl std::fmt::Display for ReleaseExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseException {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub pinned: bool,
    pub ratio: f64,
    pub rotation_days: i64,
    pub issuer: String,
    pub threshold: f64,
    pub duration_minutes: i64,
    pub status: ReleaseExceptionStatus,
}

impl ReleaseException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseExceptionStatus) -> bool {
        match self.status {
            ReleaseExceptionStatus::Idle => {
                match next {
                    ReleaseExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseExceptionStatus::Warming => {
                match next {
                    ReleaseExceptionStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseExceptionStatus::Ready => {
                match next {
                    ReleaseExceptionStatus::Draining | ReleaseExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseExceptionStatus::Draining => {
                match next {
                    ReleaseExceptionStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ReleaseExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.duration_minutes as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseException {
        ReleaseException {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            pinned: self.pinned,
            ratio: self.ratio,
            rotation_days: self.rotation_days,
            issuer: self.issuer.clone(),
            threshold: self.threshold,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a release exception in status idle.
pub fn new_release_exception(code: &str, name: &str) -> ReleaseException {
    ReleaseException {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        pinned: false,
        ratio: 0.0,
        rotation_days: 0,
        issuer: "".to_string(),
        threshold: 0.0,
        duration_minutes: 0,
        status: ReleaseExceptionStatus::Idle,
    }
}
