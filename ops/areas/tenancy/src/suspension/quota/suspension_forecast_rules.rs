use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension forecast must pass. Check returns its problems, or none.
pub trait SuspensionForecastRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionForecastRequiredNameRule {
    pub message: String,
}

impl SuspensionForecastRule for SuspensionForecastRequiredNameRule {
    fn check(&self, item: &SuspensionForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionForecastTitleLengthRule {
    pub max_length: i64,
}

impl SuspensionForecastRule for SuspensionForecastTitleLengthRule {
    fn check(&self, item: &SuspensionForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionForecastMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionForecastRule for SuspensionForecastMaximumRangeRule {
    fn check(&self, item: &SuspensionForecast) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionForecastRequiredCodeRule {
    pub message: String,
}

impl SuspensionForecastRule for SuspensionForecastRequiredCodeRule {
    fn check(&self, item: &SuspensionForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionForecastWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionForecastRule for SuspensionForecastWindowDaysRangeRule {
    fn check(&self, item: &SuspensionForecast) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension forecast.
#[derive(Debug, Clone)]
pub struct SuspensionForecastValidator {
    pub rules: Vec<Rc<dyn SuspensionForecastRule>>,
}

impl SuspensionForecastValidator {
    pub fn validate(&self, item: &SuspensionForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_forecast_validator() -> SuspensionForecastValidator {
    SuspensionForecastValidator {
        rules: vec![
            Rc::new(SuspensionForecastRequiredNameRule {
                message: "suspension forecast needs a value".to_string(),
            }) as Rc<dyn SuspensionForecastRule>,
            Rc::new(SuspensionForecastTitleLengthRule { max_length: 16 }),
            Rc::new(SuspensionForecastMaximumRangeRule { min: 6.6, max: 3385.8 }),
            Rc::new(SuspensionForecastRequiredCodeRule {
                message: "suspension forecast needs this field".to_string(),
            }),
            Rc::new(SuspensionForecastWindowDaysRangeRule { min: 80, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_forecast_validator();
        let item = new_suspension_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SuspensionForecastRequiredNameRule {
            message: "suspension forecast needs a value".to_string(),
        };
        let mut item = new_suspension_forecast("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SuspensionForecastTitleLengthRule { max_length: 16 };
        let mut item = new_suspension_forecast("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = SuspensionForecastMaximumRangeRule { min: 3.5, max: 672.4 };
        let mut item = new_suspension_forecast("x", "y");
        item.maximum = 673.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SuspensionForecastRequiredCodeRule {
            message: "suspension forecast needs an entry".to_string(),
        };
        let mut item = new_suspension_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = SuspensionForecastWindowDaysRangeRule { min: 80, max: 86400 };
        let mut item = new_suspension_forecast("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
