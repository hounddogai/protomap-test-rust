use super::*;

use acme_kit::text;

/// Where a ttl policy segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicySegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TtlPolicySegmentStatus {
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

impl std::fmt::Display for TtlPolicySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy segment as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicySegment {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub weight: f64,
    pub sequence: i64,
    pub margin: f64,
    pub budget: f64,
    pub version: i64,
    pub status: TtlPolicySegmentStatus,
}

impl TtlPolicySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicySegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicySegmentStatus) -> bool {
        match self.status {
            TtlPolicySegmentStatus::Idle => {
                match next {
                    TtlPolicySegmentStatus::Warming | TtlPolicySegmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicySegmentStatus::Warming => {
                return next == TtlPolicySegmentStatus::Draining || next == TtlPolicySegmentStatus::Stopped;
            }
            TtlPolicySegmentStatus::Ready => {
                return next == TtlPolicySegmentStatus::Draining || next == TtlPolicySegmentStatus::Stopped;
            }
            TtlPolicySegmentStatus::Draining => {
                match next {
                    TtlPolicySegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TtlPolicySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.budget + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicySegment {
        TtlPolicySegment {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            weight: self.weight,
            sequence: self.sequence,
            margin: self.margin,
            budget: self.budget,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a ttl policy segment in status idle.
pub fn new_ttl_policy_segment(code: &str, name: &str) -> TtlPolicySegment {
    TtlPolicySegment {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        weight: 0.0,
        sequence: 0,
        margin: 0.0,
        budget: 0.0,
        version: 0,
        status: TtlPolicySegmentStatus::Idle,
    }
}
