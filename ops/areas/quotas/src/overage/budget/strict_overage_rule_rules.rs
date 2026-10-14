use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict overage rule must pass. Check returns its problems, or none.
pub trait StrictOverageRuleRule: std::fmt::Debug {
    fn check(&self, item: &StrictOverageRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageRuleCodeLengthRule {
    pub max_length: i64,
}

impl StrictOverageRuleRule for StrictOverageRuleCodeLengthRule {
    fn check(&self, item: &StrictOverageRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageRuleApprovedGuardRule {
    pub limit: i64,
}

impl StrictOverageRuleRule for StrictOverageRuleApprovedGuardRule {
    fn check(&self, item: &StrictOverageRule) -> Vec<String> {
        if item.approved && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while approved",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictOverageRuleRequiredNameRule {
    pub message: String,
}

impl StrictOverageRuleRule for StrictOverageRuleRequiredNameRule {
    fn check(&self, item: &StrictOverageRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict overage rule.
#[derive(Debug, Clone)]
pub struct StrictOverageRuleValidator {
    pub rules: Vec<Rc<dyn StrictOverageRuleRule>>,
}

impl StrictOverageRuleValidator {
    pub fn validate(&self, item: &StrictOverageRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictOverageRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictOverageRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_overage_rule_validator() -> StrictOverageRuleValidator {
    StrictOverageRuleValidator {
        rules: vec![
            Rc::new(StrictOverageRuleCodeLengthRule { max_length: 1000 }) as Rc<dyn StrictOverageRuleRule>,
            Rc::new(StrictOverageRuleApprovedGuardRule { limit: 1000 }),
            Rc::new(StrictOverageRuleRequiredNameRule {
                message: "strict overage rule needs this field".to_string(),
            }),
        ],
    }
}
