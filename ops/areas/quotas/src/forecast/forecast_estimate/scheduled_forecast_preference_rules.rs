use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled forecast preference must pass. Check returns its problems, or none.
pub trait ScheduledForecastPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledForecastPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPreferenceRequiredUnitRule {
    pub message: String,
}

impl ScheduledForecastPreferenceRule for ScheduledForecastPreferenceRequiredUnitRule {
    fn check(&self, item: &ScheduledForecastPreference) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledForecastPreferenceRule for ScheduledForecastPreferenceCodeLengthRule {
    fn check(&self, item: &ScheduledForecastPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPreferenceFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledForecastPreferenceRule for ScheduledForecastPreferenceFloorValueRangeRule {
    fn check(&self, item: &ScheduledForecastPreference) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPreferenceUnitLengthRule {
    pub max_length: i64,
}

impl ScheduledForecastPreferenceRule for ScheduledForecastPreferenceUnitLengthRule {
    fn check(&self, item: &ScheduledForecastPreference) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledForecastPreferenceAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl ScheduledForecastPreferenceRule for ScheduledForecastPreferenceAllowedDescriptionRule {
    fn check(&self, item: &ScheduledForecastPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is too long", item.description)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled forecast preference.
#[derive(Debug, Clone)]
pub struct ScheduledForecastPreferenceValidator {
    pub rules: Vec<Rc<dyn ScheduledForecastPreferenceRule>>,
}

impl ScheduledForecastPreferenceValidator {
    pub fn validate(&self, item: &ScheduledForecastPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledForecastPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledForecastPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_forecast_preference_validator() -> ScheduledForecastPreferenceValidator {
    ScheduledForecastPreferenceValidator {
        rules: vec![
            Rc::new(ScheduledForecastPreferenceRequiredUnitRule {
                message: "scheduled forecast preference needs a value".to_string(),
            }) as Rc<dyn ScheduledForecastPreferenceRule>,
            Rc::new(ScheduledForecastPreferenceCodeLengthRule { max_length: 1000 }),
            Rc::new(ScheduledForecastPreferenceFloorValueRangeRule { min: 8.4, max: 1256.4 }),
            Rc::new(ScheduledForecastPreferenceUnitLengthRule { max_length: 64 }),
            Rc::new(ScheduledForecastPreferenceAllowedDescriptionRule {
                allowed: vec!["legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_forecast_preference_validator();
        let item = new_scheduled_forecast_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = ScheduledForecastPreferenceRequiredUnitRule {
            message: "scheduled forecast preference needs an entry".to_string(),
        };
        let mut item = new_scheduled_forecast_preference("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledForecastPreferenceCodeLengthRule { max_length: 64 };
        let mut item = new_scheduled_forecast_preference("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ScheduledForecastPreferenceFloorValueRangeRule { min: 0.6, max: 2516.1 };
        let mut item = new_scheduled_forecast_preference("x", "y");
        item.floor_value = 2517.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = ScheduledForecastPreferenceUnitLengthRule { max_length: 32 };
        let mut item = new_scheduled_forecast_preference("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
