use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy measurement baseline must pass. Check returns its problems, or none.
pub trait LegacyMeasurementBaselineRule: std::fmt::Debug {
    fn check(&self, item: &LegacyMeasurementBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBaselineTimezoneLengthRule {
    pub max_length: i64,
}

impl LegacyMeasurementBaselineRule for LegacyMeasurementBaselineTimezoneLengthRule {
    fn check(&self, item: &LegacyMeasurementBaseline) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBaselinePrincipalLengthRule {
    pub max_length: i64,
}

impl LegacyMeasurementBaselineRule for LegacyMeasurementBaselinePrincipalLengthRule {
    fn check(&self, item: &LegacyMeasurementBaseline) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBaselineThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyMeasurementBaselineRule for LegacyMeasurementBaselineThresholdRangeRule {
    fn check(&self, item: &LegacyMeasurementBaseline) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBaselineVisibleGuardRule {
    pub limit: f64,
}

impl LegacyMeasurementBaselineRule for LegacyMeasurementBaselineVisibleGuardRule {
    fn check(&self, item: &LegacyMeasurementBaseline) -> Vec<String> {
        if item.visible && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while visible", item.score, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy measurement baseline.
#[derive(Debug, Clone)]
pub struct LegacyMeasurementBaselineValidator {
    pub rules: Vec<Rc<dyn LegacyMeasurementBaselineRule>>,
}

impl LegacyMeasurementBaselineValidator {
    pub fn validate(&self, item: &LegacyMeasurementBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyMeasurementBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyMeasurementBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_measurement_baseline_validator() -> LegacyMeasurementBaselineValidator {
    LegacyMeasurementBaselineValidator {
        rules: vec![
            Rc::new(LegacyMeasurementBaselineTimezoneLengthRule { max_length: 80 })
                as Rc<dyn LegacyMeasurementBaselineRule>,
            Rc::new(LegacyMeasurementBaselinePrincipalLengthRule { max_length: 3600 }),
            Rc::new(LegacyMeasurementBaselineThresholdRangeRule { min: 9.4, max: 4227.6 }),
            Rc::new(LegacyMeasurementBaselineVisibleGuardRule { limit: 3.2 }),
        ],
    }
}
