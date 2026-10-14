use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global overage threshold must pass. Check returns its problems, or none.
pub trait GlobalOverageThresholdRule: std::fmt::Debug {
    fn check(&self, item: &GlobalOverageThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOverageThresholdScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalOverageThresholdRule for GlobalOverageThresholdScoreRangeRule {
    fn check(&self, item: &GlobalOverageThreshold) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOverageThresholdActiveGuardRule {
    pub limit: f64,
}

impl GlobalOverageThresholdRule for GlobalOverageThresholdActiveGuardRule {
    fn check(&self, item: &GlobalOverageThreshold) -> Vec<String> {
        if item.active && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while active", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOverageThresholdFingerprintLengthRule {
    pub max_length: i64,
}

impl GlobalOverageThresholdRule for GlobalOverageThresholdFingerprintLengthRule {
    fn check(&self, item: &GlobalOverageThreshold) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global overage threshold.
#[derive(Debug, Clone)]
pub struct GlobalOverageThresholdValidator {
    pub rules: Vec<Rc<dyn GlobalOverageThresholdRule>>,
}

impl GlobalOverageThresholdValidator {
    pub fn validate(&self, item: &GlobalOverageThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalOverageThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalOverageThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_overage_threshold_validator() -> GlobalOverageThresholdValidator {
    GlobalOverageThresholdValidator {
        rules: vec![
            Rc::new(GlobalOverageThresholdScoreRangeRule { min: 2.1, max: 601.5 })
                as Rc<dyn GlobalOverageThresholdRule>,
            Rc::new(GlobalOverageThresholdActiveGuardRule { limit: 9.3 }),
            Rc::new(GlobalOverageThresholdFingerprintLengthRule { max_length: 64 }),
        ],
    }
}
