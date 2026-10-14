use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback rollup must pass. Check returns its problems, or none.
pub trait RollbackRollupRule: std::fmt::Debug {
    fn check(&self, item: &RollbackRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRollupTitleLengthRule {
    pub max_length: i64,
}

impl RollbackRollupRule for RollbackRollupTitleLengthRule {
    fn check(&self, item: &RollbackRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRollupExternalRefLengthRule {
    pub max_length: i64,
}

impl RollbackRollupRule for RollbackRollupExternalRefLengthRule {
    fn check(&self, item: &RollbackRollup) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRollupVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackRollupRule for RollbackRollupVersionRangeRule {
    fn check(&self, item: &RollbackRollup) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRollupDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollbackRollupRule for RollbackRollupDiscountRangeRule {
    fn check(&self, item: &RollbackRollup) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRollupCodePrefixRule {
    pub prefix: String,
}

impl RollbackRollupRule for RollbackRollupCodePrefixRule {
    fn check(&self, item: &RollbackRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRollupRequiredLocaleRule {
    pub message: String,
}

impl RollbackRollupRule for RollbackRollupRequiredLocaleRule {
    fn check(&self, item: &RollbackRollup) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback rollup.
#[derive(Debug, Clone)]
pub struct RollbackRollupValidator {
    pub rules: Vec<Rc<dyn RollbackRollupRule>>,
}

impl RollbackRollupValidator {
    pub fn validate(&self, item: &RollbackRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_rollup_validator() -> RollbackRollupValidator {
    RollbackRollupValidator {
        rules: vec![
            Rc::new(RollbackRollupTitleLengthRule { max_length: 255 }) as Rc<dyn RollbackRollupRule>,
            Rc::new(RollbackRollupExternalRefLengthRule { max_length: 86400 }),
            Rc::new(RollbackRollupVersionRangeRule { min: 32, max: 1000 }),
            Rc::new(RollbackRollupDiscountRangeRule { min: 1.4, max: 1141.7 }),
            Rc::new(RollbackRollupCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RollbackRollupRequiredLocaleRule {
                message: "rollback rollup needs a value".to_string(),
            }),
        ],
    }
}
