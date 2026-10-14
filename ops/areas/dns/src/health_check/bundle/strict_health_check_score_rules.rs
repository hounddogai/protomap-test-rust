use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict health check score must pass. Check returns its problems, or none.
pub trait StrictHealthCheckScoreRule: std::fmt::Debug {
    fn check(&self, item: &StrictHealthCheckScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckScoreRequiredFingerprintRule {
    pub message: String,
}

impl StrictHealthCheckScoreRule for StrictHealthCheckScoreRequiredFingerprintRule {
    fn check(&self, item: &StrictHealthCheckScore) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckScoreMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictHealthCheckScoreRule for StrictHealthCheckScoreMaximumRangeRule {
    fn check(&self, item: &StrictHealthCheckScore) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckScoreChannelLengthRule {
    pub max_length: i64,
}

impl StrictHealthCheckScoreRule for StrictHealthCheckScoreChannelLengthRule {
    fn check(&self, item: &StrictHealthCheckScore) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict health check score.
#[derive(Debug, Clone)]
pub struct StrictHealthCheckScoreValidator {
    pub rules: Vec<Rc<dyn StrictHealthCheckScoreRule>>,
}

impl StrictHealthCheckScoreValidator {
    pub fn validate(&self, item: &StrictHealthCheckScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictHealthCheckScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictHealthCheckScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_health_check_score_validator() -> StrictHealthCheckScoreValidator {
    StrictHealthCheckScoreValidator {
        rules: vec![
            Rc::new(StrictHealthCheckScoreRequiredFingerprintRule {
                message: "strict health check score needs this field".to_string(),
            }) as Rc<dyn StrictHealthCheckScoreRule>,
            Rc::new(StrictHealthCheckScoreMaximumRangeRule { min: 0.1, max: 4638.9 }),
            Rc::new(StrictHealthCheckScoreChannelLengthRule { max_length: 16 }),
        ],
    }
}
