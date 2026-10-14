use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover estimate must pass. Check returns its problems, or none.
pub trait FailoverEstimateRule: std::fmt::Debug {
    fn check(&self, item: &FailoverEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverEstimateRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverEstimateRule for FailoverEstimateRateRangeRule {
    fn check(&self, item: &FailoverEstimate) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverEstimateCipherLengthRule {
    pub max_length: i64,
}

impl FailoverEstimateRule for FailoverEstimateCipherLengthRule {
    fn check(&self, item: &FailoverEstimate) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverEstimateCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverEstimateRule for FailoverEstimateCostRangeRule {
    fn check(&self, item: &FailoverEstimate) -> Vec<String> {
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
pub struct FailoverEstimateRequiredCipherRule {
    pub message: String,
}

impl FailoverEstimateRule for FailoverEstimateRequiredCipherRule {
    fn check(&self, item: &FailoverEstimate) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverEstimateRegionLengthRule {
    pub max_length: i64,
}

impl FailoverEstimateRule for FailoverEstimateRegionLengthRule {
    fn check(&self, item: &FailoverEstimate) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverEstimateTitleLengthRule {
    pub max_length: i64,
}

impl FailoverEstimateRule for FailoverEstimateTitleLengthRule {
    fn check(&self, item: &FailoverEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover estimate.
#[derive(Debug, Clone)]
pub struct FailoverEstimateValidator {
    pub rules: Vec<Rc<dyn FailoverEstimateRule>>,
}

impl FailoverEstimateValidator {
    pub fn validate(&self, item: &FailoverEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_estimate_validator() -> FailoverEstimateValidator {
    FailoverEstimateValidator {
        rules: vec![
            Rc::new(FailoverEstimateRateRangeRule { min: 0.4, max: 3566.8 }) as Rc<dyn FailoverEstimateRule>,
            Rc::new(FailoverEstimateCipherLengthRule { max_length: 16 }),
            Rc::new(FailoverEstimateCostRangeRule { min: 5.8, max: 3641.8 }),
            Rc::new(FailoverEstimateRequiredCipherRule {
                message: "failover estimate needs this field".to_string(),
            }),
            Rc::new(FailoverEstimateRegionLengthRule { max_length: 64 }),
            Rc::new(FailoverEstimateTitleLengthRule { max_length: 3600 }),
        ],
    }
}
