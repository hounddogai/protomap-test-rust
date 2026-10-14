use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped budget burn batch must pass. Check returns its problems, or none.
pub trait CappedBudgetBurnBatchRule: std::fmt::Debug {
    fn check(&self, item: &CappedBudgetBurnBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnBatchLabelLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnBatchRule for CappedBudgetBurnBatchLabelLengthRule {
    fn check(&self, item: &CappedBudgetBurnBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnBatchChecksumLengthRule {
    pub max_length: i64,
}

impl CappedBudgetBurnBatchRule for CappedBudgetBurnBatchChecksumLengthRule {
    fn check(&self, item: &CappedBudgetBurnBatch) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnBatchRequiredFingerprintRule {
    pub message: String,
}

impl CappedBudgetBurnBatchRule for CappedBudgetBurnBatchRequiredFingerprintRule {
    fn check(&self, item: &CappedBudgetBurnBatch) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped budget burn batch.
#[derive(Debug, Clone)]
pub struct CappedBudgetBurnBatchValidator {
    pub rules: Vec<Rc<dyn CappedBudgetBurnBatchRule>>,
}

impl CappedBudgetBurnBatchValidator {
    pub fn validate(&self, item: &CappedBudgetBurnBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedBudgetBurnBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedBudgetBurnBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_budget_burn_batch_validator() -> CappedBudgetBurnBatchValidator {
    CappedBudgetBurnBatchValidator {
        rules: vec![
            Rc::new(CappedBudgetBurnBatchLabelLengthRule { max_length: 120 }) as Rc<dyn CappedBudgetBurnBatchRule>,
            Rc::new(CappedBudgetBurnBatchChecksumLengthRule { max_length: 1000 }),
            Rc::new(CappedBudgetBurnBatchRequiredFingerprintRule {
                message: "capped budget burn batch needs a value".to_string(),
            }),
        ],
    }
}
