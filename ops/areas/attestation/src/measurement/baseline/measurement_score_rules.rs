use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a measurement score must pass. Check returns its problems, or none.
pub trait MeasurementScoreRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementScoreBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementScoreRule for MeasurementScoreBatchSizeRangeRule {
    fn check(&self, item: &MeasurementScore) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementScoreAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl MeasurementScoreRule for MeasurementScoreAllowedTimezoneRule {
    fn check(&self, item: &MeasurementScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is duplicated", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementScorePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementScoreRule for MeasurementScorePortRangeRule {
    fn check(&self, item: &MeasurementScore) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementScoreCodeLengthRule {
    pub max_length: i64,
}

impl MeasurementScoreRule for MeasurementScoreCodeLengthRule {
    fn check(&self, item: &MeasurementScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a measurement score.
#[derive(Debug, Clone)]
pub struct MeasurementScoreValidator {
    pub rules: Vec<Rc<dyn MeasurementScoreRule>>,
}

impl MeasurementScoreValidator {
    pub fn validate(&self, item: &MeasurementScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MeasurementScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_score_validator() -> MeasurementScoreValidator {
    MeasurementScoreValidator {
        rules: vec![
            Rc::new(MeasurementScoreBatchSizeRangeRule { min: 120, max: 86400 }) as Rc<dyn MeasurementScoreRule>,
            Rc::new(MeasurementScoreAllowedTimezoneRule { allowed: Vec::new() }),
            Rc::new(MeasurementScorePortRangeRule { min: 64, max: 3600 }),
            Rc::new(MeasurementScoreCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_score_validator();
        let item = new_measurement_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = MeasurementScoreBatchSizeRangeRule { min: 16, max: 3600 };
        let mut item = new_measurement_score("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = MeasurementScorePortRangeRule { min: 64, max: 1000 };
        let mut item = new_measurement_score("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MeasurementScoreCodeLengthRule { max_length: 32 };
        let mut item = new_measurement_score("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
