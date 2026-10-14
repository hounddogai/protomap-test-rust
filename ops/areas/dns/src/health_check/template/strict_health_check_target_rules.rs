use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict health check target must pass. Check returns its problems, or none.
pub trait StrictHealthCheckTargetRule: std::fmt::Debug {
    fn check(&self, item: &StrictHealthCheckTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckTargetRequiredLabelRule {
    pub message: String,
}

impl StrictHealthCheckTargetRule for StrictHealthCheckTargetRequiredLabelRule {
    fn check(&self, item: &StrictHealthCheckTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckTargetRequiredCodeRule {
    pub message: String,
}

impl StrictHealthCheckTargetRule for StrictHealthCheckTargetRequiredCodeRule {
    fn check(&self, item: &StrictHealthCheckTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckTargetRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictHealthCheckTargetRule for StrictHealthCheckTargetRatioRangeRule {
    fn check(&self, item: &StrictHealthCheckTarget) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckTargetRequiredSourceRule {
    pub message: String,
}

impl StrictHealthCheckTargetRule for StrictHealthCheckTargetRequiredSourceRule {
    fn check(&self, item: &StrictHealthCheckTarget) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckTargetCodeLengthRule {
    pub max_length: i64,
}

impl StrictHealthCheckTargetRule for StrictHealthCheckTargetCodeLengthRule {
    fn check(&self, item: &StrictHealthCheckTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict health check target.
#[derive(Debug, Clone)]
pub struct StrictHealthCheckTargetValidator {
    pub rules: Vec<Rc<dyn StrictHealthCheckTargetRule>>,
}

impl StrictHealthCheckTargetValidator {
    pub fn validate(&self, item: &StrictHealthCheckTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictHealthCheckTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictHealthCheckTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_health_check_target_validator() -> StrictHealthCheckTargetValidator {
    StrictHealthCheckTargetValidator {
        rules: vec![
            Rc::new(StrictHealthCheckTargetRequiredLabelRule {
                message: "strict health check target needs a value".to_string(),
            }) as Rc<dyn StrictHealthCheckTargetRule>,
            Rc::new(StrictHealthCheckTargetRequiredCodeRule {
                message: "strict health check target needs an entry".to_string(),
            }),
            Rc::new(StrictHealthCheckTargetRatioRangeRule { min: 6.5, max: 1883.0 }),
            Rc::new(StrictHealthCheckTargetRequiredSourceRule {
                message: "strict health check target needs this field".to_string(),
            }),
            Rc::new(StrictHealthCheckTargetCodeLengthRule { max_length: 86400 }),
        ],
    }
}
