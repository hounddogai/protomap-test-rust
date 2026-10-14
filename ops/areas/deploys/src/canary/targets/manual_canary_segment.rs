use super::*;

use acme_kit::text;

/// Where a manual canary segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualCanarySegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualCanarySegmentStatus {
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

impl std::fmt::Display for ManualCanarySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual canary segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanarySegment {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub floor_value: f64,
    pub quantity: i64,
    pub grace_days: i64,
    pub version: i64,
    pub host: String,
    pub fingerprint: String,
    pub approved: bool,
    pub status: ManualCanarySegmentStatus,
}

impl ManualCanarySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualCanarySegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualCanarySegmentStatus) -> bool {
        match self.status {
            ManualCanarySegmentStatus::Idle => {
                return next == ManualCanarySegmentStatus::Draining || next == ManualCanarySegmentStatus::Stopped;
            }
            ManualCanarySegmentStatus::Warming => {
                match next {
                    ManualCanarySegmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualCanarySegmentStatus::Ready => {
                return next == ManualCanarySegmentStatus::Draining;
            }
            ManualCanarySegmentStatus::Draining => {
                match next {
                    ManualCanarySegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualCanarySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual canary segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual canary segment", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.version as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualCanarySegment {
        ManualCanarySegment {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            floor_value: self.floor_value,
            quantity: self.quantity,
            grace_days: self.grace_days,
            version: self.version,
            host: self.host.clone(),
            fingerprint: self.fingerprint.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a manual canary segment in status idle.
pub fn new_manual_canary_segment(code: &str, name: &str) -> ManualCanarySegment {
    ManualCanarySegment {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        floor_value: 0.0,
        quantity: 0,
        grace_days: 0,
        version: 0,
        host: "".to_string(),
        fingerprint: "".to_string(),
        approved: false,
        status: ManualCanarySegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_canary_segment("code-412", "name-801");
        assert_eq!(item.status, ManualCanarySegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_manual_canary_segment("code-412", "name-801");
        if item.move_to(ManualCanarySegmentStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualCanarySegmentStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_canary_segment("code-412", "name-801");
        item.status = ManualCanarySegmentStatus::Stopped;
        assert!(
            item.move_to(ManualCanarySegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
