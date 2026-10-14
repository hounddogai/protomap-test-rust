use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding rollup must pass. Check returns its problems, or none.
pub trait BindingRollupRule: std::fmt::Debug {
    fn check(&self, item: &BindingRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRollupRequiredCodeRule {
    pub message: String,
}

impl BindingRollupRule for BindingRollupRequiredCodeRule {
    fn check(&self, item: &BindingRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRollupMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BindingRollupRule for BindingRollupMinimumRangeRule {
    fn check(&self, item: &BindingRollup) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingRollupNameLengthRule {
    pub max_length: i64,
}

impl BindingRollupRule for BindingRollupNameLengthRule {
    fn check(&self, item: &BindingRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding rollup.
#[derive(Debug, Clone)]
pub struct BindingRollupValidator {
    pub rules: Vec<Rc<dyn BindingRollupRule>>,
}

impl BindingRollupValidator {
    pub fn validate(&self, item: &BindingRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_rollup_validator() -> BindingRollupValidator {
    BindingRollupValidator {
        rules: vec![
            Rc::new(BindingRollupRequiredCodeRule {
                message: "binding rollup needs a value".to_string(),
            }) as Rc<dyn BindingRollupRule>,
            Rc::new(BindingRollupMinimumRangeRule { min: 5.8, max: 3125.6 }),
            Rc::new(BindingRollupNameLengthRule { max_length: 16 }),
        ],
    }
}
