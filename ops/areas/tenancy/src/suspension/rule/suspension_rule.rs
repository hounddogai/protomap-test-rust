use super::*;

use acme_kit::text;

/// Where a suspension rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SuspensionRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SuspensionRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension rule as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRule {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub grace_days: i64,
    pub audience: String,
    pub status: SuspensionRuleStatus,
}

impl SuspensionRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionRuleStatus) -> bool {
        match self.status {
            SuspensionRuleStatus::Created => {
                match next {
                    SuspensionRuleStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionRuleStatus::Validated => {
                match next {
                    SuspensionRuleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionRuleStatus::Published => {
                return next == SuspensionRuleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "suspension rule", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.grace_days as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionRule {
        SuspensionRule {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            grace_days: self.grace_days,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a suspension rule in status created.
pub fn new_suspension_rule(code: &str, name: &str) -> SuspensionRule {
    SuspensionRule {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        grace_days: 0,
        audience: "".to_string(),
        status: SuspensionRuleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_suspension_rule("code-704", "name-225");
        assert_eq!(item.status, SuspensionRuleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_suspension_rule("code-704", "name-225");
        if item.move_to(SuspensionRuleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionRuleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_suspension_rule("code-704", "name-225");
        item.status = SuspensionRuleStatus::Retired;
        assert!(
            item.move_to(SuspensionRuleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
