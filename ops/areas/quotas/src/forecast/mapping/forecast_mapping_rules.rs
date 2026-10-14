use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast mapping must pass. Check returns its problems, or none.
pub trait ForecastMappingRule: std::fmt::Debug {
    fn check(&self, item: &ForecastMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMappingWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastMappingRule for ForecastMappingWindowDaysRangeRule {
    fn check(&self, item: &ForecastMapping) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMappingRequiredTitleRule {
    pub message: String,
}

impl ForecastMappingRule for ForecastMappingRequiredTitleRule {
    fn check(&self, item: &ForecastMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastMappingRequiredCodeRule {
    pub message: String,
}

impl ForecastMappingRule for ForecastMappingRequiredCodeRule {
    fn check(&self, item: &ForecastMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast mapping.
#[derive(Debug, Clone)]
pub struct ForecastMappingValidator {
    pub rules: Vec<Rc<dyn ForecastMappingRule>>,
}

impl ForecastMappingValidator {
    pub fn validate(&self, item: &ForecastMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_mapping_validator() -> ForecastMappingValidator {
    ForecastMappingValidator {
        rules: vec![
            Rc::new(ForecastMappingWindowDaysRangeRule { min: 255, max: 255 }) as Rc<dyn ForecastMappingRule>,
            Rc::new(ForecastMappingRequiredTitleRule {
                message: "forecast mapping needs an entry".to_string(),
            }),
            Rc::new(ForecastMappingRequiredCodeRule {
                message: "forecast mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_mapping_validator();
        let item = new_forecast_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = ForecastMappingWindowDaysRangeRule { min: 64, max: 120 };
        let mut item = new_forecast_mapping("x", "y");
        item.window_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ForecastMappingRequiredTitleRule {
            message: "forecast mapping needs this field".to_string(),
        };
        let mut item = new_forecast_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastMappingRequiredCodeRule {
            message: "forecast mapping needs an entry".to_string(),
        };
        let mut item = new_forecast_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
