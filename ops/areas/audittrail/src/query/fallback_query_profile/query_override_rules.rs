use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a query override must pass. Check returns its problems, or none.
pub trait QueryOverrideRule: std::fmt::Debug {
    fn check(&self, item: &QueryOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryOverrideCategoryLengthRule {
    pub max_length: i64,
}

impl QueryOverrideRule for QueryOverrideCategoryLengthRule {
    fn check(&self, item: &QueryOverride) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryOverrideCodeLengthRule {
    pub max_length: i64,
}

impl QueryOverrideRule for QueryOverrideCodeLengthRule {
    fn check(&self, item: &QueryOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryOverrideSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QueryOverrideRule for QueryOverrideSequenceRangeRule {
    fn check(&self, item: &QueryOverride) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query override.
#[derive(Debug, Clone)]
pub struct QueryOverrideValidator {
    pub rules: Vec<Rc<dyn QueryOverrideRule>>,
}

impl QueryOverrideValidator {
    pub fn validate(&self, item: &QueryOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QueryOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_override_validator() -> QueryOverrideValidator {
    QueryOverrideValidator {
        rules: vec![
            Rc::new(QueryOverrideCategoryLengthRule { max_length: 3600 }) as Rc<dyn QueryOverrideRule>,
            Rc::new(QueryOverrideCodeLengthRule { max_length: 64 }),
            Rc::new(QueryOverrideSequenceRangeRule { min: 16, max: 32 }),
        ],
    }
}
