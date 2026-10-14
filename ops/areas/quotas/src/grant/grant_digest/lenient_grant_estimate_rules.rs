use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient grant estimate must pass. Check returns its problems, or none.
pub trait LenientGrantEstimateRule: std::fmt::Debug {
    fn check(&self, item: &LenientGrantEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantEstimateCodePrefixRule {
    pub prefix: String,
}

impl LenientGrantEstimateRule for LenientGrantEstimateCodePrefixRule {
    fn check(&self, item: &LenientGrantEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantEstimateRequiredCodeRule {
    pub message: String,
}

impl LenientGrantEstimateRule for LenientGrantEstimateRequiredCodeRule {
    fn check(&self, item: &LenientGrantEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantEstimateTimezoneLengthRule {
    pub max_length: i64,
}

impl LenientGrantEstimateRule for LenientGrantEstimateTimezoneLengthRule {
    fn check(&self, item: &LenientGrantEstimate) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantEstimateSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientGrantEstimateRule for LenientGrantEstimateSequenceRangeRule {
    fn check(&self, item: &LenientGrantEstimate) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientGrantEstimateLabelLengthRule {
    pub max_length: i64,
}

impl LenientGrantEstimateRule for LenientGrantEstimateLabelLengthRule {
    fn check(&self, item: &LenientGrantEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient grant estimate.
#[derive(Debug, Clone)]
pub struct LenientGrantEstimateValidator {
    pub rules: Vec<Rc<dyn LenientGrantEstimateRule>>,
}

impl LenientGrantEstimateValidator {
    pub fn validate(&self, item: &LenientGrantEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientGrantEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientGrantEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_grant_estimate_validator() -> LenientGrantEstimateValidator {
    LenientGrantEstimateValidator {
        rules: vec![
            Rc::new(LenientGrantEstimateCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn LenientGrantEstimateRule>,
            Rc::new(LenientGrantEstimateRequiredCodeRule {
                message: "lenient grant estimate needs an entry".to_string(),
            }),
            Rc::new(LenientGrantEstimateTimezoneLengthRule { max_length: 16 }),
            Rc::new(LenientGrantEstimateSequenceRangeRule { min: 64, max: 255 }),
            Rc::new(LenientGrantEstimateLabelLengthRule { max_length: 64 }),
        ],
    }
}
