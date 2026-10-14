use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension target must pass. Check returns its problems, or none.
pub trait SuspensionTargetRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTargetWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionTargetRule for SuspensionTargetWeightRangeRule {
    fn check(&self, item: &SuspensionTarget) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTargetLabelLengthRule {
    pub max_length: i64,
}

impl SuspensionTargetRule for SuspensionTargetLabelLengthRule {
    fn check(&self, item: &SuspensionTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTargetRequiredCodeRule {
    pub message: String,
}

impl SuspensionTargetRule for SuspensionTargetRequiredCodeRule {
    fn check(&self, item: &SuspensionTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension target.
#[derive(Debug, Clone)]
pub struct SuspensionTargetValidator {
    pub rules: Vec<Rc<dyn SuspensionTargetRule>>,
}

impl SuspensionTargetValidator {
    pub fn validate(&self, item: &SuspensionTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_target_validator() -> SuspensionTargetValidator {
    SuspensionTargetValidator {
        rules: vec![
            Rc::new(SuspensionTargetWeightRangeRule { min: 4.2, max: 3634.7 }) as Rc<dyn SuspensionTargetRule>,
            Rc::new(SuspensionTargetLabelLengthRule { max_length: 86400 }),
            Rc::new(SuspensionTargetRequiredCodeRule {
                message: "suspension target needs an entry".to_string(),
            }),
        ],
    }
}
