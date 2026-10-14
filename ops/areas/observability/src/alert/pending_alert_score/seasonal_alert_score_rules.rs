use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal alert score must pass. Check returns its problems, or none.
pub trait SeasonalAlertScoreRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalAlertScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertScoreCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalAlertScoreRule for SeasonalAlertScoreCodeLengthRule {
    fn check(&self, item: &SeasonalAlertScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertScoreRequiredTitleRule {
    pub message: String,
}

impl SeasonalAlertScoreRule for SeasonalAlertScoreRequiredTitleRule {
    fn check(&self, item: &SeasonalAlertScore) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertScoreRequiredUnitRule {
    pub message: String,
}

impl SeasonalAlertScoreRule for SeasonalAlertScoreRequiredUnitRule {
    fn check(&self, item: &SeasonalAlertScore) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertScoreLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalAlertScoreRule for SeasonalAlertScoreLimitRangeRule {
    fn check(&self, item: &SeasonalAlertScore) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertScoreVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalAlertScoreRule for SeasonalAlertScoreVersionRangeRule {
    fn check(&self, item: &SeasonalAlertScore) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertScoreRequiredCodeRule {
    pub message: String,
}

impl SeasonalAlertScoreRule for SeasonalAlertScoreRequiredCodeRule {
    fn check(&self, item: &SeasonalAlertScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal alert score.
#[derive(Debug, Clone)]
pub struct SeasonalAlertScoreValidator {
    pub rules: Vec<Rc<dyn SeasonalAlertScoreRule>>,
}

impl SeasonalAlertScoreValidator {
    pub fn validate(&self, item: &SeasonalAlertScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalAlertScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalAlertScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_alert_score_validator() -> SeasonalAlertScoreValidator {
    SeasonalAlertScoreValidator {
        rules: vec![
            Rc::new(SeasonalAlertScoreCodeLengthRule { max_length: 16 }) as Rc<dyn SeasonalAlertScoreRule>,
            Rc::new(SeasonalAlertScoreRequiredTitleRule {
                message: "seasonal alert score needs a value".to_string(),
            }),
            Rc::new(SeasonalAlertScoreRequiredUnitRule {
                message: "seasonal alert score needs a value".to_string(),
            }),
            Rc::new(SeasonalAlertScoreLimitRangeRule { min: 80, max: 255 }),
            Rc::new(SeasonalAlertScoreVersionRangeRule { min: 16, max: 1000 }),
            Rc::new(SeasonalAlertScoreRequiredCodeRule {
                message: "seasonal alert score needs an entry".to_string(),
            }),
        ],
    }
}
