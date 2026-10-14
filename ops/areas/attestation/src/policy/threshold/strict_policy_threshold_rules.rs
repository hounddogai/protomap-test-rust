use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict policy threshold must pass. Check returns its problems, or none.
pub trait StrictPolicyThresholdRule: std::fmt::Debug {
    fn check(&self, item: &StrictPolicyThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyThresholdRequiredNameRule {
    pub message: String,
}

impl StrictPolicyThresholdRule for StrictPolicyThresholdRequiredNameRule {
    fn check(&self, item: &StrictPolicyThreshold) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyThresholdAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictPolicyThresholdRule for StrictPolicyThresholdAmountRangeRule {
    fn check(&self, item: &StrictPolicyThreshold) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyThresholdThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictPolicyThresholdRule for StrictPolicyThresholdThresholdRangeRule {
    fn check(&self, item: &StrictPolicyThreshold) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyThresholdRequiredCodeRule {
    pub message: String,
}

impl StrictPolicyThresholdRule for StrictPolicyThresholdRequiredCodeRule {
    fn check(&self, item: &StrictPolicyThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyThresholdDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictPolicyThresholdRule for StrictPolicyThresholdDurationMinutesRangeRule {
    fn check(&self, item: &StrictPolicyThreshold) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a strict policy threshold.
#[derive(Debug, Clone)]
pub struct StrictPolicyThresholdValidator {
    pub rules: Vec<Rc<dyn StrictPolicyThresholdRule>>,
}

impl StrictPolicyThresholdValidator {
    pub fn validate(&self, item: &StrictPolicyThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictPolicyThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictPolicyThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_policy_threshold_validator() -> StrictPolicyThresholdValidator {
    StrictPolicyThresholdValidator {
        rules: vec![
            Rc::new(StrictPolicyThresholdRequiredNameRule {
                message: "strict policy threshold needs this field".to_string(),
            }) as Rc<dyn StrictPolicyThresholdRule>,
            Rc::new(StrictPolicyThresholdAmountRangeRule { min: 4.3, max: 122.4 }),
            Rc::new(StrictPolicyThresholdThresholdRangeRule { min: 0.2, max: 811.6 }),
            Rc::new(StrictPolicyThresholdRequiredCodeRule {
                message: "strict policy threshold needs a value".to_string(),
            }),
            Rc::new(StrictPolicyThresholdDurationMinutesRangeRule { min: 120, max: 3600 }),
        ],
    }
}
