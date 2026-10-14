use super::*;

use acme_kit::text;

/// Where a default failover rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFailoverRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultFailoverRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultFailoverRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default failover rule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverRule {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub quantity: i64,
    pub weight: f64,
    pub status: DefaultFailoverRuleStatus,
}

impl DefaultFailoverRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultFailoverRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultFailoverRuleStatus) -> bool {
        match self.status {
            DefaultFailoverRuleStatus::Requested => {
                match next {
                    DefaultFailoverRuleStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFailoverRuleStatus::Granted => {
                return next == DefaultFailoverRuleStatus::Revoked || next == DefaultFailoverRuleStatus::Expired;
            }
            DefaultFailoverRuleStatus::Revoked => {
                return next == DefaultFailoverRuleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultFailoverRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default failover rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quantity as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultFailoverRule {
        DefaultFailoverRule {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            quantity: self.quantity,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default failover rule in status requested.
pub fn new_default_failover_rule(code: &str, name: &str) -> DefaultFailoverRule {
    DefaultFailoverRule {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        quantity: 0,
        weight: 0.0,
        status: DefaultFailoverRuleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_failover_rule("code-201", "name-729");
        assert_eq!(item.status, DefaultFailoverRuleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_default_failover_rule("code-201", "name-729");
        if item.move_to(DefaultFailoverRuleStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultFailoverRuleStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_failover_rule("code-201", "name-729");
        item.status = DefaultFailoverRuleStatus::Expired;
        assert!(
            item.move_to(DefaultFailoverRuleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
