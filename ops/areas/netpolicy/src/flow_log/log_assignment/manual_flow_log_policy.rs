use super::*;

use acme_kit::text;

/// Where a manual flow log policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualFlowLogPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualFlowLogPolicyStatus {
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

impl std::fmt::Display for ManualFlowLogPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual flow log policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFlowLogPolicy {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub ttl_seconds: i64,
    pub revision: i64,
    pub sequence: i64,
    pub status: ManualFlowLogPolicyStatus,
}

impl ManualFlowLogPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualFlowLogPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualFlowLogPolicyStatus) -> bool {
        match self.status {
            ManualFlowLogPolicyStatus::Idle => {
                return next == ManualFlowLogPolicyStatus::Ready;
            }
            ManualFlowLogPolicyStatus::Warming => {
                return next == ManualFlowLogPolicyStatus::Draining;
            }
            ManualFlowLogPolicyStatus::Ready => {
                match next {
                    ManualFlowLogPolicyStatus::Draining | ManualFlowLogPolicyStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualFlowLogPolicyStatus::Draining => {
                return next == ManualFlowLogPolicyStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualFlowLogPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual flow log policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.revision as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualFlowLogPolicy {
        ManualFlowLogPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            ttl_seconds: self.ttl_seconds,
            revision: self.revision,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual flow log policy in status idle.
pub fn new_manual_flow_log_policy(code: &str, label: &str) -> ManualFlowLogPolicy {
    ManualFlowLogPolicy {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        ttl_seconds: 0,
        revision: 0,
        sequence: 0,
        status: ManualFlowLogPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_flow_log_policy("code-743", "label-879");
        assert_eq!(item.status, ManualFlowLogPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_manual_flow_log_policy("code-743", "label-879");
        if item.move_to(ManualFlowLogPolicyStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualFlowLogPolicyStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_flow_log_policy("code-743", "label-879");
        item.status = ManualFlowLogPolicyStatus::Stopped;
        assert!(
            item.move_to(ManualFlowLogPolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
