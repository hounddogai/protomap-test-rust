use super::*;

use acme_kit::text;

/// Where a capped flow log rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedFlowLogRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedFlowLogRuleStatus {
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

impl std::fmt::Display for CappedFlowLogRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped flow log rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedFlowLogRule {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub key_id: String,
    pub cost: f64,
    pub phone: String,
    pub status: CappedFlowLogRuleStatus,
}

impl CappedFlowLogRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedFlowLogRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedFlowLogRuleStatus) -> bool {
        match self.status {
            CappedFlowLogRuleStatus::Idle => {
                match next {
                    CappedFlowLogRuleStatus::Ready | CappedFlowLogRuleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedFlowLogRuleStatus::Warming => {
                return next == CappedFlowLogRuleStatus::Ready || next == CappedFlowLogRuleStatus::Stopped;
            }
            CappedFlowLogRuleStatus::Ready => {
                return next == CappedFlowLogRuleStatus::Draining || next == CappedFlowLogRuleStatus::Stopped;
            }
            CappedFlowLogRuleStatus::Draining => {
                return next == CappedFlowLogRuleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedFlowLogRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped flow log rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> CappedFlowLogRule {
        CappedFlowLogRule {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            key_id: self.key_id.clone(),
            cost: self.cost,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a capped flow log rule in status idle.
pub fn new_capped_flow_log_rule(code: &str, title: &str) -> CappedFlowLogRule {
    CappedFlowLogRule {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        key_id: "".to_string(),
        cost: 0.0,
        phone: "".to_string(),
        status: CappedFlowLogRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_capped_flow_log_rule("code-185", "title-312");
        assert_eq!(item.status, CappedFlowLogRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_capped_flow_log_rule("code-185", "title-312");
        if item.move_to(CappedFlowLogRuleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedFlowLogRuleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_capped_flow_log_rule("code-185", "title-312");
        item.status = CappedFlowLogRuleStatus::Stopped;
        assert!(
            item.move_to(CappedFlowLogRuleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
