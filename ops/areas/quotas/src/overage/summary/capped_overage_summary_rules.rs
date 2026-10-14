use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped overage summary must pass. Check returns its problems, or none.
pub trait CappedOverageSummaryRule: std::fmt::Debug {
    fn check(&self, item: &CappedOverageSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSummaryCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedOverageSummaryRule for CappedOverageSummaryCostRangeRule {
    fn check(&self, item: &CappedOverageSummary) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSummaryCodePrefixRule {
    pub prefix: String,
}

impl CappedOverageSummaryRule for CappedOverageSummaryCodePrefixRule {
    fn check(&self, item: &CappedOverageSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSummaryQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedOverageSummaryRule for CappedOverageSummaryQuantityRangeRule {
    fn check(&self, item: &CappedOverageSummary) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSummaryRequiredCodeRule {
    pub message: String,
}

impl CappedOverageSummaryRule for CappedOverageSummaryRequiredCodeRule {
    fn check(&self, item: &CappedOverageSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped overage summary.
#[derive(Debug, Clone)]
pub struct CappedOverageSummaryValidator {
    pub rules: Vec<Rc<dyn CappedOverageSummaryRule>>,
}

impl CappedOverageSummaryValidator {
    pub fn validate(&self, item: &CappedOverageSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedOverageSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedOverageSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_overage_summary_validator() -> CappedOverageSummaryValidator {
    CappedOverageSummaryValidator {
        rules: vec![
            Rc::new(CappedOverageSummaryCostRangeRule { min: 3.7, max: 2467.3 }) as Rc<dyn CappedOverageSummaryRule>,
            Rc::new(CappedOverageSummaryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CappedOverageSummaryQuantityRangeRule { min: 255, max: 86400 }),
            Rc::new(CappedOverageSummaryRequiredCodeRule {
                message: "capped overage summary needs a value".to_string(),
            }),
        ],
    }
}
