use super::*;

use acme_kit::text;

/// Where a manual identity rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualIdentityRuleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualIdentityRuleStatus {
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

impl std::fmt::Display for ManualIdentityRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual identity rule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualIdentityRule {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub quantity: i64,
    pub host: String,
    pub status: ManualIdentityRuleStatus,
}

impl ManualIdentityRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualIdentityRuleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualIdentityRuleStatus) -> bool {
        match self.status {
            ManualIdentityRuleStatus::New => {
                return next == ManualIdentityRuleStatus::Queued;
            }
            ManualIdentityRuleStatus::Queued => {
                match next {
                    ManualIdentityRuleStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIdentityRuleStatus::Running => {
                match next {
                    ManualIdentityRuleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualIdentityRuleStatus::Done => {
                return next == ManualIdentityRuleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualIdentityRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual identity rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.cost - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualIdentityRule {
        ManualIdentityRule {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            quantity: self.quantity,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual identity rule in status new.
pub fn new_manual_identity_rule(code: &str, name: &str) -> ManualIdentityRule {
    ManualIdentityRule {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        quantity: 0,
        host: "".to_string(),
        status: ManualIdentityRuleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_manual_identity_rule("code-63", "name-175");
        assert_eq!(item.status, ManualIdentityRuleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_manual_identity_rule("code-63", "name-175");
        if item.move_to(ManualIdentityRuleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualIdentityRuleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_manual_identity_rule("code-63", "name-175");
        item.status = ManualIdentityRuleStatus::Failed;
        assert!(
            item.move_to(ManualIdentityRuleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
