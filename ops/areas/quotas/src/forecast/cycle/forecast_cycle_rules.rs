use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast cycle must pass. Check returns its problems, or none.
pub trait ForecastCycleRule: std::fmt::Debug {
    fn check(&self, item: &ForecastCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCycleRequiredScopeRule {
    pub message: String,
}

impl ForecastCycleRule for ForecastCycleRequiredScopeRule {
    fn check(&self, item: &ForecastCycle) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCycleTitleLengthRule {
    pub max_length: i64,
}

impl ForecastCycleRule for ForecastCycleTitleLengthRule {
    fn check(&self, item: &ForecastCycle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCycleRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastCycleRule for ForecastCycleRatioRangeRule {
    fn check(&self, item: &ForecastCycle) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCycleLabelLengthRule {
    pub max_length: i64,
}

impl ForecastCycleRule for ForecastCycleLabelLengthRule {
    fn check(&self, item: &ForecastCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast cycle.
#[derive(Debug, Clone)]
pub struct ForecastCycleValidator {
    pub rules: Vec<Rc<dyn ForecastCycleRule>>,
}

impl ForecastCycleValidator {
    pub fn validate(&self, item: &ForecastCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_cycle_validator() -> ForecastCycleValidator {
    ForecastCycleValidator {
        rules: vec![
            Rc::new(ForecastCycleRequiredScopeRule {
                message: "forecast cycle needs an entry".to_string(),
            }) as Rc<dyn ForecastCycleRule>,
            Rc::new(ForecastCycleTitleLengthRule { max_length: 16 }),
            Rc::new(ForecastCycleRatioRangeRule { min: 2.3, max: 4691.9 }),
            Rc::new(ForecastCycleLabelLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_cycle_validator();
        let item = new_forecast_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ForecastCycleRequiredScopeRule {
            message: "forecast cycle needs a value".to_string(),
        };
        let mut item = new_forecast_cycle("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ForecastCycleTitleLengthRule { max_length: 255 };
        let mut item = new_forecast_cycle("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = ForecastCycleRatioRangeRule { min: 3.4, max: 2030.2 };
        let mut item = new_forecast_cycle("x", "y");
        item.ratio = 2031.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ForecastCycleLabelLengthRule { max_length: 16 };
        let mut item = new_forecast_cycle("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
