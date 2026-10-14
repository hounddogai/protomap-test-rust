use super::*;

use acme_kit::text;

/// Where a policy rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PolicyRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PolicyRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyRule {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub phone: String,
    pub grace_days: i64,
    pub starts_at: i64,
    pub active: bool,
    pub priority: i64,
    pub status: PolicyRuleStatus,
}

impl PolicyRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyRuleStatus) -> bool {
        match self.status {
            PolicyRuleStatus::New => {
                match next {
                    PolicyRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyRuleStatus::Queued => {
                match next {
                    PolicyRuleStatus::Done | PolicyRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyRuleStatus::Running => {
                match next {
                    PolicyRuleStatus::Done | PolicyRuleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyRuleStatus::Done => {
                return next == PolicyRuleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "policy rule", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.grace_days as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyRule {
        PolicyRule {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            phone: self.phone.clone(),
            grace_days: self.grace_days,
            starts_at: self.starts_at,
            active: self.active,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a policy rule in status new.
pub fn new_policy_rule(code: &str, name: &str) -> PolicyRule {
    PolicyRule {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        phone: "".to_string(),
        grace_days: 0,
        starts_at: 0,
        active: false,
        priority: 0,
        status: PolicyRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_policy_rule("code-241", "name-85");
        assert_eq!(item.status, PolicyRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_policy_rule("code-241", "name-85");
        if item.move_to(PolicyRuleStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyRuleStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_policy_rule("code-241", "name-85");
        item.status = PolicyRuleStatus::Failed;
        assert!(item.move_to(PolicyRuleStatus::New).is_err(), "expected move to fail");
    }
}
