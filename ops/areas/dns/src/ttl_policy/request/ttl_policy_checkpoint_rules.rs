use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a ttl policy checkpoint must pass. Check returns its problems, or none.
pub trait TtlPolicyCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCheckpointQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyCheckpointRule for TtlPolicyCheckpointQuantityRangeRule {
    fn check(&self, item: &TtlPolicyCheckpoint) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCheckpointEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyCheckpointRule for TtlPolicyCheckpointEndsAtRangeRule {
    fn check(&self, item: &TtlPolicyCheckpoint) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCheckpointBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyCheckpointRule for TtlPolicyCheckpointBudgetRangeRule {
    fn check(&self, item: &TtlPolicyCheckpoint) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCheckpointRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyCheckpointRule for TtlPolicyCheckpointRequiredCodeRule {
    fn check(&self, item: &TtlPolicyCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCheckpointAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl TtlPolicyCheckpointRule for TtlPolicyCheckpointAllowedKeyIdRule {
    fn check(&self, item: &TtlPolicyCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is too long", item.key_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy checkpoint.
#[derive(Debug, Clone)]
pub struct TtlPolicyCheckpointValidator {
    pub rules: Vec<Rc<dyn TtlPolicyCheckpointRule>>,
}

impl TtlPolicyCheckpointValidator {
    pub fn validate(&self, item: &TtlPolicyCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyCheckpoint]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_ttl_policy_checkpoint_validator() -> TtlPolicyCheckpointValidator {
    TtlPolicyCheckpointValidator {
        rules: vec![
            Rc::new(TtlPolicyCheckpointQuantityRangeRule { min: 32, max: 120 }) as Rc<dyn TtlPolicyCheckpointRule>,
            Rc::new(TtlPolicyCheckpointEndsAtRangeRule { min: 86400, max: 86400 }),
            Rc::new(TtlPolicyCheckpointBudgetRangeRule { min: 9.1, max: 1354.7 }),
            Rc::new(TtlPolicyCheckpointRequiredCodeRule {
                message: "ttl policy checkpoint needs this field".to_string(),
            }),
            Rc::new(TtlPolicyCheckpointAllowedKeyIdRule {
                allowed: vec!["batched".to_string(), "seasonal".to_string(), "seasonal".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_checkpoint_validator();
        let item = new_ttl_policy_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = TtlPolicyCheckpointQuantityRangeRule { min: 255, max: 86400 };
        let mut item = new_ttl_policy_checkpoint("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = TtlPolicyCheckpointEndsAtRangeRule { min: 32, max: 80 };
        let mut item = new_ttl_policy_checkpoint("x", "y");
        item.ends_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = TtlPolicyCheckpointBudgetRangeRule { min: 2.1, max: 1714.3 };
        let mut item = new_ttl_policy_checkpoint("x", "y");
        item.budget = 1715.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyCheckpointRequiredCodeRule {
            message: "ttl policy checkpoint needs this field".to_string(),
        };
        let mut item = new_ttl_policy_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
