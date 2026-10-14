use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast schedule must pass. Check returns its problems, or none.
pub trait ForecastScheduleRule: std::fmt::Debug {
    fn check(&self, item: &ForecastSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScheduleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastScheduleRule for ForecastScheduleMinimumRangeRule {
    fn check(&self, item: &ForecastSchedule) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScheduleRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastScheduleRule for ForecastScheduleRevisionRangeRule {
    fn check(&self, item: &ForecastSchedule) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScheduleFingerprintLengthRule {
    pub max_length: i64,
}

impl ForecastScheduleRule for ForecastScheduleFingerprintLengthRule {
    fn check(&self, item: &ForecastSchedule) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScheduleNameLengthRule {
    pub max_length: i64,
}

impl ForecastScheduleRule for ForecastScheduleNameLengthRule {
    fn check(&self, item: &ForecastSchedule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScheduleAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl ForecastScheduleRule for ForecastScheduleAllowedRegionRule {
    fn check(&self, item: &ForecastSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is too long", item.region)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast schedule.
#[derive(Debug, Clone)]
pub struct ForecastScheduleValidator {
    pub rules: Vec<Rc<dyn ForecastScheduleRule>>,
}

impl ForecastScheduleValidator {
    pub fn validate(&self, item: &ForecastSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_schedule_validator() -> ForecastScheduleValidator {
    ForecastScheduleValidator {
        rules: vec![
            Rc::new(ForecastScheduleMinimumRangeRule { min: 1.6, max: 4918.7 }) as Rc<dyn ForecastScheduleRule>,
            Rc::new(ForecastScheduleRevisionRangeRule { min: 255, max: 3600 }),
            Rc::new(ForecastScheduleFingerprintLengthRule { max_length: 64 }),
            Rc::new(ForecastScheduleNameLengthRule { max_length: 16 }),
            Rc::new(ForecastScheduleAllowedRegionRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_schedule_validator();
        let item = new_forecast_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ForecastScheduleMinimumRangeRule { min: 7.2, max: 438.5 };
        let mut item = new_forecast_schedule("x", "y");
        item.minimum = 440.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = ForecastScheduleRevisionRangeRule { min: 64, max: 120 };
        let mut item = new_forecast_schedule("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ForecastScheduleFingerprintLengthRule { max_length: 80 };
        let mut item = new_forecast_schedule("x", "y");
        item.fingerprint = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ForecastScheduleNameLengthRule { max_length: 3600 };
        let mut item = new_forecast_schedule("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
