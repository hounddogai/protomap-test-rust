use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal postmortem adjustment must pass. Check returns its problems, or none.
pub trait SeasonalPostmortemAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalPostmortemAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemAdjustmentRequiredCodeRule {
    pub message: String,
}

impl SeasonalPostmortemAdjustmentRule for SeasonalPostmortemAdjustmentRequiredCodeRule {
    fn check(&self, item: &SeasonalPostmortemAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemAdjustmentPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalPostmortemAdjustmentRule for SeasonalPostmortemAdjustmentPriorityRangeRule {
    fn check(&self, item: &SeasonalPostmortemAdjustment) -> Vec<String> {
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
pub struct SeasonalPostmortemAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalPostmortemAdjustmentRule for SeasonalPostmortemAdjustmentCodeLengthRule {
    fn check(&self, item: &SeasonalPostmortemAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemAdjustmentLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalPostmortemAdjustmentRule for SeasonalPostmortemAdjustmentLabelLengthRule {
    fn check(&self, item: &SeasonalPostmortemAdjustment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl SeasonalPostmortemAdjustmentRule for SeasonalPostmortemAdjustmentCodePrefixRule {
    fn check(&self, item: &SeasonalPostmortemAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemAdjustmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SeasonalPostmortemAdjustmentRule for SeasonalPostmortemAdjustmentAllowedCodeRule {
    fn check(&self, item: &SeasonalPostmortemAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal postmortem adjustment.
#[derive(Debug, Clone)]
pub struct SeasonalPostmortemAdjustmentValidator {
    pub rules: Vec<Rc<dyn SeasonalPostmortemAdjustmentRule>>,
}

impl SeasonalPostmortemAdjustmentValidator {
    pub fn validate(&self, item: &SeasonalPostmortemAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalPostmortemAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalPostmortemAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_postmortem_adjustment_validator() -> SeasonalPostmortemAdjustmentValidator {
    SeasonalPostmortemAdjustmentValidator {
        rules: vec![
            Rc::new(SeasonalPostmortemAdjustmentRequiredCodeRule {
                message: "seasonal postmortem adjustment needs an entry".to_string(),
            }) as Rc<dyn SeasonalPostmortemAdjustmentRule>,
            Rc::new(SeasonalPostmortemAdjustmentPriorityRangeRule { min: 64, max: 64 }),
            Rc::new(SeasonalPostmortemAdjustmentCodeLengthRule { max_length: 16 }),
            Rc::new(SeasonalPostmortemAdjustmentLabelLengthRule { max_length: 16 }),
            Rc::new(SeasonalPostmortemAdjustmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(SeasonalPostmortemAdjustmentAllowedCodeRule {
                allowed: vec!["strict".to_string(), "regional".to_string()],
            }),
        ],
    }
}
