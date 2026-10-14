use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule policy must pass. Check returns its problems, or none.
pub trait RulePolicyRule: std::fmt::Debug {
    fn check(&self, item: &RulePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePolicyNotesLengthRule {
    pub max_length: i64,
}

impl RulePolicyRule for RulePolicyNotesLengthRule {
    fn check(&self, item: &RulePolicy) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePolicyVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RulePolicyRule for RulePolicyVersionRangeRule {
    fn check(&self, item: &RulePolicy) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePolicyWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RulePolicyRule for RulePolicyWindowDaysRangeRule {
    fn check(&self, item: &RulePolicy) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePolicyAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RulePolicyRule for RulePolicyAllowedNameRule {
    fn check(&self, item: &RulePolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePolicyRequiredNotesRule {
    pub message: String,
}

impl RulePolicyRule for RulePolicyRequiredNotesRule {
    fn check(&self, item: &RulePolicy) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule policy.
#[derive(Debug, Clone)]
pub struct RulePolicyValidator {
    pub rules: Vec<Rc<dyn RulePolicyRule>>,
}

impl RulePolicyValidator {
    pub fn validate(&self, item: &RulePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RulePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RulePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_policy_validator() -> RulePolicyValidator {
    RulePolicyValidator {
        rules: vec![
            Rc::new(RulePolicyNotesLengthRule { max_length: 1000 }) as Rc<dyn RulePolicyRule>,
            Rc::new(RulePolicyVersionRangeRule { min: 255, max: 3600 }),
            Rc::new(RulePolicyWindowDaysRangeRule { min: 120, max: 3600 }),
            Rc::new(RulePolicyAllowedNameRule { allowed: Vec::new() }),
            Rc::new(RulePolicyRequiredNotesRule {
                message: "rule policy needs this field".to_string(),
            }),
        ],
    }
}
