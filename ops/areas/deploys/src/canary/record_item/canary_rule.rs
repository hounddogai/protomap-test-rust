use super::*;

use acme_kit::text;

/// Where a canary rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CanaryRuleStatus {
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

impl std::fmt::Display for CanaryRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary rule as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRule {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub timezone: String,
    pub pinned: bool,
    pub audience: String,
    pub priority: i64,
    pub external_ref: String,
    pub floor_value: f64,
    pub status: CanaryRuleStatus,
}

impl CanaryRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryRuleStatus) -> bool {
        match self.status {
            CanaryRuleStatus::New => {
                match next {
                    CanaryRuleStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryRuleStatus::Queued => {
                return next == CanaryRuleStatus::Done || next == CanaryRuleStatus::Failed;
            }
            CanaryRuleStatus::Running => {
                return next == CanaryRuleStatus::Done;
            }
            CanaryRuleStatus::Done => {
                return next == CanaryRuleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.priority as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryRule {
        CanaryRule {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            timezone: self.timezone.clone(),
            pinned: self.pinned,
            audience: self.audience.clone(),
            priority: self.priority,
            external_ref: self.external_ref.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a canary rule in status new.
pub fn new_canary_rule(code: &str, name: &str) -> CanaryRule {
    CanaryRule {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        timezone: "".to_string(),
        pinned: false,
        audience: "".to_string(),
        priority: 0,
        external_ref: "".to_string(),
        floor_value: 0.0,
        status: CanaryRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_canary_rule("code-993", "name-553");
        assert_eq!(item.status, CanaryRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_canary_rule("code-993", "name-553");
        if item.move_to(CanaryRuleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryRuleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_canary_rule("code-993", "name-553");
        item.status = CanaryRuleStatus::Failed;
        assert!(item.move_to(CanaryRuleStatus::New).is_err(), "expected move to fail");
    }
}
