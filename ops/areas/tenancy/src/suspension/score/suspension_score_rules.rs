use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension score must pass. Check returns its problems, or none.
pub trait SuspensionScoreRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScoreDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionScoreRule for SuspensionScoreDurationMinutesRangeRule {
    fn check(&self, item: &SuspensionScore) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScoreRequiredLabelRule {
    pub message: String,
}

impl SuspensionScoreRule for SuspensionScoreRequiredLabelRule {
    fn check(&self, item: &SuspensionScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScoreContactEmailLengthRule {
    pub max_length: i64,
}

impl SuspensionScoreRule for SuspensionScoreContactEmailLengthRule {
    fn check(&self, item: &SuspensionScore) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScoreThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionScoreRule for SuspensionScoreThresholdRangeRule {
    fn check(&self, item: &SuspensionScore) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionScoreRequiredDescriptionRule {
    pub message: String,
}

impl SuspensionScoreRule for SuspensionScoreRequiredDescriptionRule {
    fn check(&self, item: &SuspensionScore) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension score.
#[derive(Debug, Clone)]
pub struct SuspensionScoreValidator {
    pub rules: Vec<Rc<dyn SuspensionScoreRule>>,
}

impl SuspensionScoreValidator {
    pub fn validate(&self, item: &SuspensionScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_score_validator() -> SuspensionScoreValidator {
    SuspensionScoreValidator {
        rules: vec![
            Rc::new(SuspensionScoreDurationMinutesRangeRule { min: 1000, max: 86400 }) as Rc<dyn SuspensionScoreRule>,
            Rc::new(SuspensionScoreRequiredLabelRule {
                message: "suspension score needs an entry".to_string(),
            }),
            Rc::new(SuspensionScoreContactEmailLengthRule { max_length: 120 }),
            Rc::new(SuspensionScoreThresholdRangeRule { min: 0.2, max: 1324.2 }),
            Rc::new(SuspensionScoreRequiredDescriptionRule {
                message: "suspension score needs this field".to_string(),
            }),
        ],
    }
}
