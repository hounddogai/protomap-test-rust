use super::*;

use acme_kit::text;

/// Where a strict release policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictReleasePolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictReleasePolicyStatus {
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

impl std::fmt::Display for StrictReleasePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict release policy, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleasePolicy {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub ttl_seconds: i64,
    pub category: String,
    pub active: bool,
    pub rate: f64,
    pub fingerprint: String,
    pub key_id: String,
    pub cluster: String,
    pub status: StrictReleasePolicyStatus,
}

impl StrictReleasePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictReleasePolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictReleasePolicyStatus) -> bool {
        match self.status {
            StrictReleasePolicyStatus::Idle => {
                return next == StrictReleasePolicyStatus::Warming || next == StrictReleasePolicyStatus::Ready;
            }
            StrictReleasePolicyStatus::Warming => {
                return next == StrictReleasePolicyStatus::Ready || next == StrictReleasePolicyStatus::Draining;
            }
            StrictReleasePolicyStatus::Ready => {
                match next {
                    StrictReleasePolicyStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictReleasePolicyStatus::Draining => {
                match next {
                    StrictReleasePolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: StrictReleasePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict release policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.capacity as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictReleasePolicy {
        StrictReleasePolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            active: self.active,
            rate: self.rate,
            fingerprint: self.fingerprint.clone(),
            key_id: self.key_id.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict release policy in status idle.
pub fn new_strict_release_policy(code: &str, name: &str) -> StrictReleasePolicy {
    StrictReleasePolicy {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        ttl_seconds: 0,
        category: "".to_string(),
        active: false,
        rate: 0.0,
        fingerprint: "".to_string(),
        key_id: "".to_string(),
        cluster: "".to_string(),
        status: StrictReleasePolicyStatus::Idle,
    }
}
