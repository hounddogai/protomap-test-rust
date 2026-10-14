use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive freeze rule must pass. Check returns its problems, or none.
pub trait AdaptiveFreezeRuleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveFreezeRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeRuleAudienceLengthRule {
    pub max_length: i64,
}

impl AdaptiveFreezeRuleRule for AdaptiveFreezeRuleAudienceLengthRule {
    fn check(&self, item: &AdaptiveFreezeRule) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeRuleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveFreezeRuleRule for AdaptiveFreezeRuleEndsAtRangeRule {
    fn check(&self, item: &AdaptiveFreezeRule) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeRulePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveFreezeRuleRule for AdaptiveFreezeRulePortRangeRule {
    fn check(&self, item: &AdaptiveFreezeRule) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeRuleRequiredCodeRule {
    pub message: String,
}

impl AdaptiveFreezeRuleRule for AdaptiveFreezeRuleRequiredCodeRule {
    fn check(&self, item: &AdaptiveFreezeRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeRuleRequiredCategoryRule {
    pub message: String,
}

impl AdaptiveFreezeRuleRule for AdaptiveFreezeRuleRequiredCategoryRule {
    fn check(&self, item: &AdaptiveFreezeRule) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeRuleRequiredAudienceRule {
    pub message: String,
}

impl AdaptiveFreezeRuleRule for AdaptiveFreezeRuleRequiredAudienceRule {
    fn check(&self, item: &AdaptiveFreezeRule) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive freeze rule.
#[derive(Debug, Clone)]
pub struct AdaptiveFreezeRuleValidator {
    pub rules: Vec<Rc<dyn AdaptiveFreezeRuleRule>>,
}

impl AdaptiveFreezeRuleValidator {
    pub fn validate(&self, item: &AdaptiveFreezeRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveFreezeRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveFreezeRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_freeze_rule_validator() -> AdaptiveFreezeRuleValidator {
    AdaptiveFreezeRuleValidator {
        rules: vec![
            Rc::new(AdaptiveFreezeRuleAudienceLengthRule { max_length: 16 }) as Rc<dyn AdaptiveFreezeRuleRule>,
            Rc::new(AdaptiveFreezeRuleEndsAtRangeRule { min: 255, max: 3600 }),
            Rc::new(AdaptiveFreezeRulePortRangeRule { min: 80, max: 255 }),
            Rc::new(AdaptiveFreezeRuleRequiredCodeRule {
                message: "adaptive freeze rule needs this field".to_string(),
            }),
            Rc::new(AdaptiveFreezeRuleRequiredCategoryRule {
                message: "adaptive freeze rule needs a value".to_string(),
            }),
            Rc::new(AdaptiveFreezeRuleRequiredAudienceRule {
                message: "adaptive freeze rule needs this field".to_string(),
            }),
        ],
    }
}
