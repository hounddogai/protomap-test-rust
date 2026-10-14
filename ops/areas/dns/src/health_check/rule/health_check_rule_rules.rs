use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check rule must pass. Check returns its problems, or none.
pub trait HealthCheckRuleRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRuleTitleLengthRule {
    pub max_length: i64,
}

impl HealthCheckRuleRule for HealthCheckRuleTitleLengthRule {
    fn check(&self, item: &HealthCheckRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRuleUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckRuleRule for HealthCheckRuleUpdatedAtRangeRule {
    fn check(&self, item: &HealthCheckRule) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRuleCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckRuleRule for HealthCheckRuleCodeLengthRule {
    fn check(&self, item: &HealthCheckRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRuleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckRuleRule for HealthCheckRuleThresholdRangeRule {
    fn check(&self, item: &HealthCheckRule) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRuleRequiredTitleRule {
    pub message: String,
}

impl HealthCheckRuleRule for HealthCheckRuleRequiredTitleRule {
    fn check(&self, item: &HealthCheckRule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check rule.
#[derive(Debug, Clone)]
pub struct HealthCheckRuleValidator {
    pub rules: Vec<Rc<dyn HealthCheckRuleRule>>,
}

impl HealthCheckRuleValidator {
    pub fn validate(&self, item: &HealthCheckRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_rule_validator() -> HealthCheckRuleValidator {
    HealthCheckRuleValidator {
        rules: vec![
            Rc::new(HealthCheckRuleTitleLengthRule { max_length: 32 }) as Rc<dyn HealthCheckRuleRule>,
            Rc::new(HealthCheckRuleUpdatedAtRangeRule { min: 16, max: 3600 }),
            Rc::new(HealthCheckRuleCodeLengthRule { max_length: 86400 }),
            Rc::new(HealthCheckRuleThresholdRangeRule { min: 0.5, max: 3562.8 }),
            Rc::new(HealthCheckRuleRequiredTitleRule {
                message: "health check rule needs an entry".to_string(),
            }),
        ],
    }
}
