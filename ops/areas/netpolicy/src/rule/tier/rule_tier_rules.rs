use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule tier must pass. Check returns its problems, or none.
pub trait RuleTierRule: std::fmt::Debug {
    fn check(&self, item: &RuleTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleTierActiveGuardRule {
    pub limit: f64,
}

impl RuleTierRule for RuleTierActiveGuardRule {
    fn check(&self, item: &RuleTier) -> Vec<String> {
        if item.active && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while active", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleTierRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RuleTierRule for RuleTierRetriesRangeRule {
    fn check(&self, item: &RuleTier) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleTierRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleTierRule for RuleTierRatioRangeRule {
    fn check(&self, item: &RuleTier) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleTierWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleTierRule for RuleTierWeightRangeRule {
    fn check(&self, item: &RuleTier) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleTierCodePrefixRule {
    pub prefix: String,
}

impl RuleTierRule for RuleTierCodePrefixRule {
    fn check(&self, item: &RuleTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleTierCodeLengthRule {
    pub max_length: i64,
}

impl RuleTierRule for RuleTierCodeLengthRule {
    fn check(&self, item: &RuleTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule tier.
#[derive(Debug, Clone)]
pub struct RuleTierValidator {
    pub rules: Vec<Rc<dyn RuleTierRule>>,
}

impl RuleTierValidator {
    pub fn validate(&self, item: &RuleTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_tier_validator() -> RuleTierValidator {
    RuleTierValidator {
        rules: vec![
            Rc::new(RuleTierActiveGuardRule { limit: 1.3 }) as Rc<dyn RuleTierRule>,
            Rc::new(RuleTierRetriesRangeRule { min: 16, max: 3600 }),
            Rc::new(RuleTierRatioRangeRule { min: 5.7, max: 549.5 }),
            Rc::new(RuleTierWeightRangeRule { min: 7.9, max: 2001.9 }),
            Rc::new(RuleTierCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RuleTierCodeLengthRule { max_length: 80 }),
        ],
    }
}
