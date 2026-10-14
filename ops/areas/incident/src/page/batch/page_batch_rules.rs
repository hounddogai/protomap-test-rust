use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page batch must pass. Check returns its problems, or none.
pub trait PageBatchRule: std::fmt::Debug {
    fn check(&self, item: &PageBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBatchRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageBatchRule for PageBatchRateRangeRule {
    fn check(&self, item: &PageBatch) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBatchCodePrefixRule {
    pub prefix: String,
}

impl PageBatchRule for PageBatchCodePrefixRule {
    fn check(&self, item: &PageBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBatchRequiredUnitRule {
    pub message: String,
}

impl PageBatchRule for PageBatchRequiredUnitRule {
    fn check(&self, item: &PageBatch) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBatchRequiredIssuerRule {
    pub message: String,
}

impl PageBatchRule for PageBatchRequiredIssuerRule {
    fn check(&self, item: &PageBatch) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a page batch.
#[derive(Debug, Clone)]
pub struct PageBatchValidator {
    pub rules: Vec<Rc<dyn PageBatchRule>>,
}

impl PageBatchValidator {
    pub fn validate(&self, item: &PageBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_batch_validator() -> PageBatchValidator {
    PageBatchValidator {
        rules: vec![
            Rc::new(PageBatchRateRangeRule { min: 6.9, max: 690.6 }) as Rc<dyn PageBatchRule>,
            Rc::new(PageBatchCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PageBatchRequiredUnitRule {
                message: "page batch needs an entry".to_string(),
            }),
            Rc::new(PageBatchRequiredIssuerRule {
                message: "page batch needs a value".to_string(),
            }),
        ],
    }
}
