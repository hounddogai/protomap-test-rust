use super::*;

use acme_kit::text;

/// Where a escalation policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl EscalationPolicyStatus {
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

impl std::fmt::Display for EscalationPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPolicy {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub tenant_id: String,
    pub maximum: f64,
    pub status: EscalationPolicyStatus,
}

impl EscalationPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationPolicyStatus) -> bool {
        match self.status {
            EscalationPolicyStatus::Idle => {
                match next {
                    EscalationPolicyStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationPolicyStatus::Warming => {
                match next {
                    EscalationPolicyStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationPolicyStatus::Ready => {
                match next {
                    EscalationPolicyStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationPolicyStatus::Draining => {
                match next {
                    EscalationPolicyStatus::Stopped => {
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

    pub fn move_to(&mut self, next: EscalationPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "escalation policy", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.attempts as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationPolicy {
        EscalationPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            tenant_id: self.tenant_id.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a escalation policy in status idle.
pub fn new_escalation_policy(code: &str, name: &str) -> EscalationPolicy {
    EscalationPolicy {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        tenant_id: "".to_string(),
        maximum: 0.0,
        status: EscalationPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_escalation_policy("code-888", "name-813");
        assert_eq!(item.status, EscalationPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_escalation_policy("code-888", "name-813");
        if item.move_to(EscalationPolicyStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationPolicyStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_escalation_policy("code-888", "name-813");
        item.status = EscalationPolicyStatus::Stopped;
        assert!(
            item.move_to(EscalationPolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
