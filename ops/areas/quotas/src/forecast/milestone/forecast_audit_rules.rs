use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast audit must pass. Check returns its problems, or none.
pub trait ForecastAuditRule: std::fmt::Debug {
    fn check(&self, item: &ForecastAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAuditSummaryLengthRule {
    pub max_length: i64,
}

impl ForecastAuditRule for ForecastAuditSummaryLengthRule {
    fn check(&self, item: &ForecastAudit) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAuditRequiredCodeRule {
    pub message: String,
}

impl ForecastAuditRule for ForecastAuditRequiredCodeRule {
    fn check(&self, item: &ForecastAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAuditScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastAuditRule for ForecastAuditScoreRangeRule {
    fn check(&self, item: &ForecastAudit) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a forecast audit.
#[derive(Debug, Clone)]
pub struct ForecastAuditValidator {
    pub rules: Vec<Rc<dyn ForecastAuditRule>>,
}

impl ForecastAuditValidator {
    pub fn validate(&self, item: &ForecastAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_audit_validator() -> ForecastAuditValidator {
    ForecastAuditValidator {
        rules: vec![
            Rc::new(ForecastAuditSummaryLengthRule { max_length: 16 }) as Rc<dyn ForecastAuditRule>,
            Rc::new(ForecastAuditRequiredCodeRule {
                message: "forecast audit needs this field".to_string(),
            }),
            Rc::new(ForecastAuditScoreRangeRule { min: 6.3, max: 996.6 }),
        ],
    }
}
