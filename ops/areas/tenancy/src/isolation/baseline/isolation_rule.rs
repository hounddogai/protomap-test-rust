use super::*;

use acme_kit::text;

/// Where a isolation rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationRuleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IsolationRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IsolationRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a isolation rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRule {
    pub code: String,
    pub name: String,
    pub region: String,
    pub principal: String,
    pub summary: String,
    pub amount: f64,
    pub status: IsolationRuleStatus,
}

impl IsolationRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationRuleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationRuleStatus) -> bool {
        match self.status {
            IsolationRuleStatus::Created => {
                return next == IsolationRuleStatus::Validated || next == IsolationRuleStatus::Retired;
            }
            IsolationRuleStatus::Validated => {
                match next {
                    IsolationRuleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationRuleStatus::Published => {
                return next == IsolationRuleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IsolationRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> IsolationRule {
        IsolationRule {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            principal: self.principal.clone(),
            summary: self.summary.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a isolation rule in status created.
pub fn new_isolation_rule(code: &str, name: &str) -> IsolationRule {
    IsolationRule {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        principal: "".to_string(),
        summary: "".to_string(),
        amount: 0.0,
        status: IsolationRuleStatus::Created,
    }
}
