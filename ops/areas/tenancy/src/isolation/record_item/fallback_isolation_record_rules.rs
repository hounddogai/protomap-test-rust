use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback isolation record must pass. Check returns its problems, or none.
pub trait FallbackIsolationRecordRule: std::fmt::Debug {
    fn check(&self, item: &FallbackIsolationRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationRecordAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl FallbackIsolationRecordRule for FallbackIsolationRecordAllowedPrincipalRule {
    fn check(&self, item: &FallbackIsolationRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is below minimum", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationRecordScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackIsolationRecordRule for FallbackIsolationRecordScoreRangeRule {
    fn check(&self, item: &FallbackIsolationRecord) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationRecordTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackIsolationRecordRule for FallbackIsolationRecordTtlSecondsRangeRule {
    fn check(&self, item: &FallbackIsolationRecord) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationRecordTitleLengthRule {
    pub max_length: i64,
}

impl FallbackIsolationRecordRule for FallbackIsolationRecordTitleLengthRule {
    fn check(&self, item: &FallbackIsolationRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIsolationRecordScopeLengthRule {
    pub max_length: i64,
}

impl FallbackIsolationRecordRule for FallbackIsolationRecordScopeLengthRule {
    fn check(&self, item: &FallbackIsolationRecord) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback isolation record.
#[derive(Debug, Clone)]
pub struct FallbackIsolationRecordValidator {
    pub rules: Vec<Rc<dyn FallbackIsolationRecordRule>>,
}

impl FallbackIsolationRecordValidator {
    pub fn validate(&self, item: &FallbackIsolationRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackIsolationRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackIsolationRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_isolation_record_validator() -> FallbackIsolationRecordValidator {
    FallbackIsolationRecordValidator {
        rules: vec![
            Rc::new(FallbackIsolationRecordAllowedPrincipalRule {
                allowed: vec!["legacy".to_string()],
            }) as Rc<dyn FallbackIsolationRecordRule>,
            Rc::new(FallbackIsolationRecordScoreRangeRule { min: 8.3, max: 948.1 }),
            Rc::new(FallbackIsolationRecordTtlSecondsRangeRule { min: 80, max: 120 }),
            Rc::new(FallbackIsolationRecordTitleLengthRule { max_length: 255 }),
            Rc::new(FallbackIsolationRecordScopeLengthRule { max_length: 120 }),
        ],
    }
}
