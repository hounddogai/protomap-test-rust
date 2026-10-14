use super::*;

use acme_kit::text;

/// Where a regional nonce segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalNonceSegmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalNonceSegmentStatus {
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

impl std::fmt::Display for RegionalNonceSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional nonce segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNonceSegment {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub ratio: f64,
    pub cost: f64,
    pub burst: i64,
    pub channel: String,
    pub summary: String,
    pub description: String,
    pub status: RegionalNonceSegmentStatus,
}

impl RegionalNonceSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalNonceSegmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalNonceSegmentStatus) -> bool {
        match self.status {
            RegionalNonceSegmentStatus::Idle => {
                match next {
                    RegionalNonceSegmentStatus::Warming | RegionalNonceSegmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalNonceSegmentStatus::Warming => {
                match next {
                    RegionalNonceSegmentStatus::Draining | RegionalNonceSegmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalNonceSegmentStatus::Ready => {
                return next == RegionalNonceSegmentStatus::Stopped;
            }
            RegionalNonceSegmentStatus::Draining => {
                return next == RegionalNonceSegmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalNonceSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional nonce segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.cost * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalNonceSegment {
        RegionalNonceSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            ratio: self.ratio,
            cost: self.cost,
            burst: self.burst,
            channel: self.channel.clone(),
            summary: self.summary.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a regional nonce segment in status idle.
pub fn new_regional_nonce_segment(code: &str, name: &str) -> RegionalNonceSegment {
    RegionalNonceSegment {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        ratio: 0.0,
        cost: 0.0,
        burst: 0,
        channel: "".to_string(),
        summary: "".to_string(),
        description: "".to_string(),
        status: RegionalNonceSegmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_nonce_segment("code-949", "name-779");
        assert_eq!(item.status, RegionalNonceSegmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_regional_nonce_segment("code-949", "name-779");
        if item.move_to(RegionalNonceSegmentStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalNonceSegmentStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_nonce_segment("code-949", "name-779");
        item.status = RegionalNonceSegmentStatus::Stopped;
        assert!(
            item.move_to(RegionalNonceSegmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
