use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a isolation rule must pass. Check returns its problems, or none.
pub trait IsolationRuleRule: std::fmt::Debug {
    fn check(&self, item: &IsolationRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRuleAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl IsolationRuleRule for IsolationRuleAllowedRegionRule {
    fn check(&self, item: &IsolationRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is above maximum", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRuleRequiredRegionRule {
    pub message: String,
}

impl IsolationRuleRule for IsolationRuleRequiredRegionRule {
    fn check(&self, item: &IsolationRule) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRuleNameLengthRule {
    pub max_length: i64,
}

impl IsolationRuleRule for IsolationRuleNameLengthRule {
    fn check(&self, item: &IsolationRule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation rule.
#[derive(Debug, Clone)]
pub struct IsolationRuleValidator {
    pub rules: Vec<Rc<dyn IsolationRuleRule>>,
}

impl IsolationRuleValidator {
    pub fn validate(&self, item: &IsolationRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IsolationRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_rule_validator() -> IsolationRuleValidator {
    IsolationRuleValidator {
        rules: vec![
            Rc::new(IsolationRuleAllowedRegionRule {
                allowed: vec!["weighted".to_string(), "manual".to_string(), "lenient".to_string()],
            }) as Rc<dyn IsolationRuleRule>,
            Rc::new(IsolationRuleRequiredRegionRule {
                message: "isolation rule needs this field".to_string(),
            }),
            Rc::new(IsolationRuleNameLengthRule { max_length: 1000 }),
        ],
    }
}
