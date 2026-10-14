use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional plan rule must pass. Check returns its problems, or none.
pub trait RegionalPlanRuleRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPlanRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanRuleAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RegionalPlanRuleRule for RegionalPlanRuleAllowedTitleRule {
    fn check(&self, item: &RegionalPlanRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanRuleTitleLengthRule {
    pub max_length: i64,
}

impl RegionalPlanRuleRule for RegionalPlanRuleTitleLengthRule {
    fn check(&self, item: &RegionalPlanRule) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanRuleCodeLengthRule {
    pub max_length: i64,
}

impl RegionalPlanRuleRule for RegionalPlanRuleCodeLengthRule {
    fn check(&self, item: &RegionalPlanRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanRuleRequiredTitleRule {
    pub message: String,
}

impl RegionalPlanRuleRule for RegionalPlanRuleRequiredTitleRule {
    fn check(&self, item: &RegionalPlanRule) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional plan rule.
#[derive(Debug, Clone)]
pub struct RegionalPlanRuleValidator {
    pub rules: Vec<Rc<dyn RegionalPlanRuleRule>>,
}

impl RegionalPlanRuleValidator {
    pub fn validate(&self, item: &RegionalPlanRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalPlanRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPlanRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_plan_rule_validator() -> RegionalPlanRuleValidator {
    RegionalPlanRuleValidator {
        rules: vec![
            Rc::new(RegionalPlanRuleAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn RegionalPlanRuleRule>,
            Rc::new(RegionalPlanRuleTitleLengthRule { max_length: 255 }),
            Rc::new(RegionalPlanRuleCodeLengthRule { max_length: 3600 }),
            Rc::new(RegionalPlanRuleRequiredTitleRule {
                message: "regional plan rule needs an entry".to_string(),
            }),
        ],
    }
}
