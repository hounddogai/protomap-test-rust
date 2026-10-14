use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional rule tier must pass. Check returns its problems, or none.
pub trait RegionalRuleTierRule: std::fmt::Debug {
    fn check(&self, item: &RegionalRuleTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleTierRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalRuleTierRule for RegionalRuleTierRatioRangeRule {
    fn check(&self, item: &RegionalRuleTier) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleTierRequiredCodeRule {
    pub message: String,
}

impl RegionalRuleTierRule for RegionalRuleTierRequiredCodeRule {
    fn check(&self, item: &RegionalRuleTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleTierTitleLengthRule {
    pub max_length: i64,
}

impl RegionalRuleTierRule for RegionalRuleTierTitleLengthRule {
    fn check(&self, item: &RegionalRuleTier) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRuleTierCodePrefixRule {
    pub prefix: String,
}

impl RegionalRuleTierRule for RegionalRuleTierCodePrefixRule {
    fn check(&self, item: &RegionalRuleTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional rule tier.
#[derive(Debug, Clone)]
pub struct RegionalRuleTierValidator {
    pub rules: Vec<Rc<dyn RegionalRuleTierRule>>,
}

impl RegionalRuleTierValidator {
    pub fn validate(&self, item: &RegionalRuleTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalRuleTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalRuleTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_rule_tier_validator() -> RegionalRuleTierValidator {
    RegionalRuleTierValidator {
        rules: vec![
            Rc::new(RegionalRuleTierRatioRangeRule { min: 5.2, max: 2169.6 }) as Rc<dyn RegionalRuleTierRule>,
            Rc::new(RegionalRuleTierRequiredCodeRule {
                message: "regional rule tier needs a value".to_string(),
            }),
            Rc::new(RegionalRuleTierTitleLengthRule { max_length: 64 }),
            Rc::new(RegionalRuleTierCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
