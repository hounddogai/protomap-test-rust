use super::*;

use acme_kit::text;

/// Where a flow log digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogDigestStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FlowLogDigestStatus {
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

impl std::fmt::Display for FlowLogDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a flow log digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDigest {
    pub code: String,
    pub label: String,
    pub host: String,
    pub phone: String,
    pub segment: String,
    pub capacity: i64,
    pub status: FlowLogDigestStatus,
}

impl FlowLogDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogDigestStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogDigestStatus) -> bool {
        match self.status {
            FlowLogDigestStatus::Idle => {
                return next == FlowLogDigestStatus::Ready || next == FlowLogDigestStatus::Draining;
            }
            FlowLogDigestStatus::Warming => {
                return next == FlowLogDigestStatus::Draining;
            }
            FlowLogDigestStatus::Ready => {
                match next {
                    FlowLogDigestStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogDigestStatus::Draining => {
                match next {
                    FlowLogDigestStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FlowLogDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> FlowLogDigest {
        FlowLogDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            host: self.host.clone(),
            phone: self.phone.clone(),
            segment: self.segment.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a flow log digest in status idle.
pub fn new_flow_log_digest(code: &str, label: &str) -> FlowLogDigest {
    FlowLogDigest {
        code: code.to_string(),
        label: label.to_string(),
        host: "".to_string(),
        phone: "".to_string(),
        segment: "".to_string(),
        capacity: 0,
        status: FlowLogDigestStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_flow_log_digest("code-890", "label-344");
        assert_eq!(item.status, FlowLogDigestStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_flow_log_digest("code-890", "label-344");
        if item.move_to(FlowLogDigestStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FlowLogDigestStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_flow_log_digest("code-890", "label-344");
        item.status = FlowLogDigestStatus::Stopped;
        assert!(
            item.move_to(FlowLogDigestStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
