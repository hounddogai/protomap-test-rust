use super::*;

use acme_kit::text;

/// Where a regional flow log segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalFlowLogSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalFlowLogSegmentStatus {
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

impl std::fmt::Display for RegionalFlowLogSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional flow log segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFlowLogSegment {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub timezone: String,
    pub window_days: i64,
    pub key_id: String,
    pub audience: String,
    pub status: RegionalFlowLogSegmentStatus,
}

impl RegionalFlowLogSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalFlowLogSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalFlowLogSegmentStatus) -> bool {
        match self.status {
            RegionalFlowLogSegmentStatus::Idle => {
                return next == RegionalFlowLogSegmentStatus::Draining;
            }
            RegionalFlowLogSegmentStatus::Warming => {
                match next {
                    RegionalFlowLogSegmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalFlowLogSegmentStatus::Ready => {
                return next == RegionalFlowLogSegmentStatus::Stopped;
            }
            RegionalFlowLogSegmentStatus::Draining => {
                match next {
                    RegionalFlowLogSegmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalFlowLogSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional flow log segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.window_days as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalFlowLogSegment {
        RegionalFlowLogSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            timezone: self.timezone.clone(),
            window_days: self.window_days,
            key_id: self.key_id.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional flow log segment in status idle.
pub fn new_regional_flow_log_segment(code: &str, name: &str) -> RegionalFlowLogSegment {
    RegionalFlowLogSegment {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        timezone: "".to_string(),
        window_days: 0,
        key_id: "".to_string(),
        audience: "".to_string(),
        status: RegionalFlowLogSegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_flow_log_segment("code-131", "name-566");
        assert_eq!(item.status, RegionalFlowLogSegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_regional_flow_log_segment("code-131", "name-566");
        if item.move_to(RegionalFlowLogSegmentStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalFlowLogSegmentStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_flow_log_segment("code-131", "name-566");
        item.status = RegionalFlowLogSegmentStatus::Stopped;
        assert!(
            item.move_to(RegionalFlowLogSegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
