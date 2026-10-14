use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal egress batch must pass. Check returns its problems, or none.
pub trait SeasonalEgressBatchRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalEgressBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressBatchRequiredTitleRule {
    pub message: String,
}

impl SeasonalEgressBatchRule for SeasonalEgressBatchRequiredTitleRule {
    fn check(&self, item: &SeasonalEgressBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressBatchCodePrefixRule {
    pub prefix: String,
}

impl SeasonalEgressBatchRule for SeasonalEgressBatchCodePrefixRule {
    fn check(&self, item: &SeasonalEgressBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressBatchRequiredCurrencyRule {
    pub message: String,
}

impl SeasonalEgressBatchRule for SeasonalEgressBatchRequiredCurrencyRule {
    fn check(&self, item: &SeasonalEgressBatch) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal egress batch.
#[derive(Debug, Clone)]
pub struct SeasonalEgressBatchValidator {
    pub rules: Vec<Rc<dyn SeasonalEgressBatchRule>>,
}

impl SeasonalEgressBatchValidator {
    pub fn validate(&self, item: &SeasonalEgressBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalEgressBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalEgressBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_egress_batch_validator() -> SeasonalEgressBatchValidator {
    SeasonalEgressBatchValidator {
        rules: vec![
            Rc::new(SeasonalEgressBatchRequiredTitleRule {
                message: "seasonal egress batch needs this field".to_string(),
            }) as Rc<dyn SeasonalEgressBatchRule>,
            Rc::new(SeasonalEgressBatchCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SeasonalEgressBatchRequiredCurrencyRule {
                message: "seasonal egress batch needs a value".to_string(),
            }),
        ],
    }
}
