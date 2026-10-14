use super::*;

use acme_kit::text;

/// Where a ttl policy rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TtlPolicyRuleStatus {
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

impl std::fmt::Display for TtlPolicyRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyRule {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub approved: bool,
    pub sequence: i64,
    pub status: TtlPolicyRuleStatus,
}

impl TtlPolicyRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyRuleStatus) -> bool {
        match self.status {
            TtlPolicyRuleStatus::Idle => {
                match next {
                    TtlPolicyRuleStatus::Draining | TtlPolicyRuleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyRuleStatus::Warming => {
                match next {
                    TtlPolicyRuleStatus::Ready | TtlPolicyRuleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyRuleStatus::Ready => {
                return next == TtlPolicyRuleStatus::Draining || next == TtlPolicyRuleStatus::Stopped;
            }
            TtlPolicyRuleStatus::Draining => {
                return next == TtlPolicyRuleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> TtlPolicyRule {
        TtlPolicyRule {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            approved: self.approved,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a ttl policy rule in status idle.
pub fn new_ttl_policy_rule(code: &str, label: &str) -> TtlPolicyRule {
    TtlPolicyRule {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        approved: false,
        sequence: 0,
        status: TtlPolicyRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_ttl_policy_rule("code-571", "label-736");
        assert_eq!(item.status, TtlPolicyRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_ttl_policy_rule("code-571", "label-736");
        if item.move_to(TtlPolicyRuleStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyRuleStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_ttl_policy_rule("code-571", "label-736");
        item.status = TtlPolicyRuleStatus::Stopped;
        assert!(
            item.move_to(TtlPolicyRuleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
