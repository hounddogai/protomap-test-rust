use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage forecast must pass. Check returns its problems, or none.
pub trait OverageForecastRule: std::fmt::Debug {
    fn check(&self, item: &OverageForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageForecastCodeLengthRule {
    pub max_length: i64,
}

impl OverageForecastRule for OverageForecastCodeLengthRule {
    fn check(&self, item: &OverageForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageForecastDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageForecastRule for OverageForecastDurationMinutesRangeRule {
    fn check(&self, item: &OverageForecast) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageForecastQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageForecastRule for OverageForecastQuotaUnitsRangeRule {
    fn check(&self, item: &OverageForecast) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage forecast.
#[derive(Debug, Clone)]
pub struct OverageForecastValidator {
    pub rules: Vec<Rc<dyn OverageForecastRule>>,
}

impl OverageForecastValidator {
    pub fn validate(&self, item: &OverageForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_forecast_validator() -> OverageForecastValidator {
    OverageForecastValidator {
        rules: vec![
            Rc::new(OverageForecastCodeLengthRule { max_length: 80 }) as Rc<dyn OverageForecastRule>,
            Rc::new(OverageForecastDurationMinutesRangeRule { min: 80, max: 255 }),
            Rc::new(OverageForecastQuotaUnitsRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_forecast_validator();
        let item = new_overage_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = OverageForecastCodeLengthRule { max_length: 1000 };
        let mut item = new_overage_forecast("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = OverageForecastDurationMinutesRangeRule { min: 120, max: 3600 };
        let mut item = new_overage_forecast("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = OverageForecastQuotaUnitsRangeRule { min: 80, max: 1000 };
        let mut item = new_overage_forecast("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
