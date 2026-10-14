use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary data export profile must pass. Check returns its problems, or none.
pub trait PrimaryDataExportProfileRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryDataExportProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportProfileAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryDataExportProfileRule for PrimaryDataExportProfileAllowedCodeRule {
    fn check(&self, item: &PrimaryDataExportProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportProfileRequiredTitleRule {
    pub message: String,
}

impl PrimaryDataExportProfileRule for PrimaryDataExportProfileRequiredTitleRule {
    fn check(&self, item: &PrimaryDataExportProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportProfileThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryDataExportProfileRule for PrimaryDataExportProfileThresholdRangeRule {
    fn check(&self, item: &PrimaryDataExportProfile) -> Vec<String> {
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
pub struct PrimaryDataExportProfileLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryDataExportProfileRule for PrimaryDataExportProfileLimitRangeRule {
    fn check(&self, item: &PrimaryDataExportProfile) -> Vec<String> {
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

/// Runs every rule of a primary data export profile.
#[derive(Debug, Clone)]
pub struct PrimaryDataExportProfileValidator {
    pub rules: Vec<Rc<dyn PrimaryDataExportProfileRule>>,
}

impl PrimaryDataExportProfileValidator {
    pub fn validate(&self, item: &PrimaryDataExportProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryDataExportProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryDataExportProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_data_export_profile_validator() -> PrimaryDataExportProfileValidator {
    PrimaryDataExportProfileValidator {
        rules: vec![
            Rc::new(PrimaryDataExportProfileAllowedCodeRule { allowed: Vec::new() })
                as Rc<dyn PrimaryDataExportProfileRule>,
            Rc::new(PrimaryDataExportProfileRequiredTitleRule {
                message: "primary data export profile needs a value".to_string(),
            }),
            Rc::new(PrimaryDataExportProfileThresholdRangeRule { min: 4.5, max: 165.5 }),
            Rc::new(PrimaryDataExportProfileLimitRangeRule { min: 32, max: 255 }),
        ],
    }
}
