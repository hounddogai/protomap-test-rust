use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive page variance must pass. Check returns its problems, or none.
pub trait AdaptivePageVarianceRule: std::fmt::Debug {
    fn check(&self, item: &AdaptivePageVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageVarianceRequiredCodeRule {
    pub message: String,
}

impl AdaptivePageVarianceRule for AdaptivePageVarianceRequiredCodeRule {
    fn check(&self, item: &AdaptivePageVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageVarianceUnitLengthRule {
    pub max_length: i64,
}

impl AdaptivePageVarianceRule for AdaptivePageVarianceUnitLengthRule {
    fn check(&self, item: &AdaptivePageVariance) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageVarianceSummaryLengthRule {
    pub max_length: i64,
}

impl AdaptivePageVarianceRule for AdaptivePageVarianceSummaryLengthRule {
    fn check(&self, item: &AdaptivePageVariance) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive page variance.
#[derive(Debug, Clone)]
pub struct AdaptivePageVarianceValidator {
    pub rules: Vec<Rc<dyn AdaptivePageVarianceRule>>,
}

impl AdaptivePageVarianceValidator {
    pub fn validate(&self, item: &AdaptivePageVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptivePageVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptivePageVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_page_variance_validator() -> AdaptivePageVarianceValidator {
    AdaptivePageVarianceValidator {
        rules: vec![
            Rc::new(AdaptivePageVarianceRequiredCodeRule {
                message: "adaptive page variance needs an entry".to_string(),
            }) as Rc<dyn AdaptivePageVarianceRule>,
            Rc::new(AdaptivePageVarianceUnitLengthRule { max_length: 255 }),
            Rc::new(AdaptivePageVarianceSummaryLengthRule { max_length: 3600 }),
        ],
    }
}
