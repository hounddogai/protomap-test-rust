use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast draft must pass. Check returns its problems, or none.
pub trait ForecastDraftRule: std::fmt::Debug {
    fn check(&self, item: &ForecastDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDraftCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastDraftRule for ForecastDraftCreatedAtRangeRule {
    fn check(&self, item: &ForecastDraft) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDraftCodePrefixRule {
    pub prefix: String,
}

impl ForecastDraftRule for ForecastDraftCodePrefixRule {
    fn check(&self, item: &ForecastDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDraftArchivedGuardRule {
    pub limit: i64,
}

impl ForecastDraftRule for ForecastDraftArchivedGuardRule {
    fn check(&self, item: &ForecastDraft) -> Vec<String> {
        if item.archived && (item.quota_units > self.limit) {
            return vec![format!(
                "quota units {} exceeds {} while archived",
                item.quota_units, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDraftCodeLengthRule {
    pub max_length: i64,
}

impl ForecastDraftRule for ForecastDraftCodeLengthRule {
    fn check(&self, item: &ForecastDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastDraftNameLengthRule {
    pub max_length: i64,
}

impl ForecastDraftRule for ForecastDraftNameLengthRule {
    fn check(&self, item: &ForecastDraft) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast draft.
#[derive(Debug, Clone)]
pub struct ForecastDraftValidator {
    pub rules: Vec<Rc<dyn ForecastDraftRule>>,
}

impl ForecastDraftValidator {
    pub fn validate(&self, item: &ForecastDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_draft_validator() -> ForecastDraftValidator {
    ForecastDraftValidator {
        rules: vec![
            Rc::new(ForecastDraftCreatedAtRangeRule { min: 16, max: 120 }) as Rc<dyn ForecastDraftRule>,
            Rc::new(ForecastDraftCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ForecastDraftArchivedGuardRule { limit: 64 }),
            Rc::new(ForecastDraftCodeLengthRule { max_length: 86400 }),
            Rc::new(ForecastDraftNameLengthRule { max_length: 1000 }),
        ],
    }
}
