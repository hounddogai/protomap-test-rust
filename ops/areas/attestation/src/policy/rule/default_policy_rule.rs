use super::*;

use acme_kit::text;

/// Where a default policy rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPolicyRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultPolicyRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultPolicyRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default policy rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyRule {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub ratio: f64,
    pub cipher: String,
    pub capacity: i64,
    pub window_days: i64,
    pub burst: i64,
    pub quota_units: i64,
    pub scope: String,
    pub status: DefaultPolicyRuleStatus,
}

impl DefaultPolicyRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPolicyRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPolicyRuleStatus) -> bool {
        match self.status {
            DefaultPolicyRuleStatus::Proposed => {
                match next {
                    DefaultPolicyRuleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPolicyRuleStatus::Approved => {
                return next == DefaultPolicyRuleStatus::Rejected;
            }
            DefaultPolicyRuleStatus::Rejected => {
                return next == DefaultPolicyRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultPolicyRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default policy rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default policy rule", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        (self.score * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultPolicyRule {
        DefaultPolicyRule {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            ratio: self.ratio,
            cipher: self.cipher.clone(),
            capacity: self.capacity,
            window_days: self.window_days,
            burst: self.burst,
            quota_units: self.quota_units,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default policy rule in status proposed.
pub fn new_default_policy_rule(code: &str, name: &str) -> DefaultPolicyRule {
    DefaultPolicyRule {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        ratio: 0.0,
        cipher: "".to_string(),
        capacity: 0,
        window_days: 0,
        burst: 0,
        quota_units: 0,
        scope: "".to_string(),
        status: DefaultPolicyRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_default_policy_rule("code-209", "name-649");
        assert_eq!(item.status, DefaultPolicyRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_default_policy_rule("code-209", "name-649");
        if item.move_to(DefaultPolicyRuleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultPolicyRuleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_default_policy_rule("code-209", "name-649");
        item.status = DefaultPolicyRuleStatus::Withdrawn;
        assert!(
            item.move_to(DefaultPolicyRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
