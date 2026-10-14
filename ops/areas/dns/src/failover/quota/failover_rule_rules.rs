use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover rule must pass. Check returns its problems, or none.
pub trait FailoverRuleRule: std::fmt::Debug {
    fn check(&self, item: &FailoverRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRuleCodeLengthRule {
    pub max_length: i64,
}

impl FailoverRuleRule for FailoverRuleCodeLengthRule {
    fn check(&self, item: &FailoverRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRuleRequiredNameRule {
    pub message: String,
}

impl FailoverRuleRule for FailoverRuleRequiredNameRule {
    fn check(&self, item: &FailoverRule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverRuleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverRuleRule for FailoverRuleBurstRangeRule {
    fn check(&self, item: &FailoverRule) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a failover rule.
#[derive(Debug, Clone)]
pub struct FailoverRuleValidator {
    pub rules: Vec<Rc<dyn FailoverRuleRule>>,
}

impl FailoverRuleValidator {
    pub fn validate(&self, item: &FailoverRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_rule_validator() -> FailoverRuleValidator {
    FailoverRuleValidator {
        rules: vec![
            Rc::new(FailoverRuleCodeLengthRule { max_length: 86400 }) as Rc<dyn FailoverRuleRule>,
            Rc::new(FailoverRuleRequiredNameRule {
                message: "failover rule needs an entry".to_string(),
            }),
            Rc::new(FailoverRuleBurstRangeRule { min: 255, max: 1000 }),
        ],
    }
}
