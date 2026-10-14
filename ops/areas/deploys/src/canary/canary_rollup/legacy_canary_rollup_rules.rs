use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy canary rollup must pass. Check returns its problems, or none.
pub trait LegacyCanaryRollupRule: std::fmt::Debug {
    fn check(&self, item: &LegacyCanaryRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryRollupTitleLengthRule {
    pub max_length: i64,
}

impl LegacyCanaryRollupRule for LegacyCanaryRollupTitleLengthRule {
    fn check(&self, item: &LegacyCanaryRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryRollupRequiredTitleRule {
    pub message: String,
}

impl LegacyCanaryRollupRule for LegacyCanaryRollupRequiredTitleRule {
    fn check(&self, item: &LegacyCanaryRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryRollupCodePrefixRule {
    pub prefix: String,
}

impl LegacyCanaryRollupRule for LegacyCanaryRollupCodePrefixRule {
    fn check(&self, item: &LegacyCanaryRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryRollupPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyCanaryRollupRule for LegacyCanaryRollupPriorityRangeRule {
    fn check(&self, item: &LegacyCanaryRollup) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryRollupUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyCanaryRollupRule for LegacyCanaryRollupUpdatedAtRangeRule {
    fn check(&self, item: &LegacyCanaryRollup) -> Vec<String> {
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

/// Runs every rule of a legacy canary rollup.
#[derive(Debug, Clone)]
pub struct LegacyCanaryRollupValidator {
    pub rules: Vec<Rc<dyn LegacyCanaryRollupRule>>,
}

impl LegacyCanaryRollupValidator {
    pub fn validate(&self, item: &LegacyCanaryRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyCanaryRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyCanaryRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_canary_rollup_validator() -> LegacyCanaryRollupValidator {
    LegacyCanaryRollupValidator {
        rules: vec![
            Rc::new(LegacyCanaryRollupTitleLengthRule { max_length: 3600 }) as Rc<dyn LegacyCanaryRollupRule>,
            Rc::new(LegacyCanaryRollupRequiredTitleRule {
                message: "legacy canary rollup needs this field".to_string(),
            }),
            Rc::new(LegacyCanaryRollupCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LegacyCanaryRollupPriorityRangeRule { min: 64, max: 1000 }),
            Rc::new(LegacyCanaryRollupUpdatedAtRangeRule { min: 255, max: 255 }),
        ],
    }
}
