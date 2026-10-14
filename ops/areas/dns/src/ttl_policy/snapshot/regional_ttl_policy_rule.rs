use super::*;

use acme_kit::text;

/// Where a regional ttl policy rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalTtlPolicyRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalTtlPolicyRuleStatus {
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

impl std::fmt::Display for RegionalTtlPolicyRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional ttl policy rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyRule {
    pub code: String,
    pub title: String,
    pub host: String,
    pub discount: f64,
    pub expires_at: i64,
    pub status: RegionalTtlPolicyRuleStatus,
}

impl RegionalTtlPolicyRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalTtlPolicyRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalTtlPolicyRuleStatus) -> bool {
        match self.status {
            RegionalTtlPolicyRuleStatus::Idle => {
                return next == RegionalTtlPolicyRuleStatus::Warming;
            }
            RegionalTtlPolicyRuleStatus::Warming => {
                return next == RegionalTtlPolicyRuleStatus::Draining || next == RegionalTtlPolicyRuleStatus::Stopped;
            }
            RegionalTtlPolicyRuleStatus::Ready => {
                return next == RegionalTtlPolicyRuleStatus::Draining;
            }
            RegionalTtlPolicyRuleStatus::Draining => {
                match next {
                    RegionalTtlPolicyRuleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalTtlPolicyRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional ttl policy rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "regional ttl policy rule", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.discount * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalTtlPolicyRule {
        RegionalTtlPolicyRule {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            discount: self.discount,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional ttl policy rule in status idle.
pub fn new_regional_ttl_policy_rule(code: &str, title: &str) -> RegionalTtlPolicyRule {
    RegionalTtlPolicyRule {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        discount: 0.0,
        expires_at: 0,
        status: RegionalTtlPolicyRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_ttl_policy_rule("code-240", "title-800");
        assert_eq!(item.status, RegionalTtlPolicyRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_regional_ttl_policy_rule("code-240", "title-800");
        if item.move_to(RegionalTtlPolicyRuleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalTtlPolicyRuleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_ttl_policy_rule("code-240", "title-800");
        item.status = RegionalTtlPolicyRuleStatus::Stopped;
        assert!(
            item.move_to(RegionalTtlPolicyRuleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
