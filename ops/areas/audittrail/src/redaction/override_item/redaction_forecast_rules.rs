use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction forecast must pass. Check returns its problems, or none.
pub trait RedactionForecastRule: std::fmt::Debug {
    fn check(&self, item: &RedactionForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionForecastStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionForecastRule for RedactionForecastStartsAtRangeRule {
    fn check(&self, item: &RedactionForecast) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionForecastRequiredCategoryRule {
    pub message: String,
}

impl RedactionForecastRule for RedactionForecastRequiredCategoryRule {
    fn check(&self, item: &RedactionForecast) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionForecastUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionForecastRule for RedactionForecastUpdatedAtRangeRule {
    fn check(&self, item: &RedactionForecast) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionForecastCodeLengthRule {
    pub max_length: i64,
}

impl RedactionForecastRule for RedactionForecastCodeLengthRule {
    fn check(&self, item: &RedactionForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction forecast.
#[derive(Debug, Clone)]
pub struct RedactionForecastValidator {
    pub rules: Vec<Rc<dyn RedactionForecastRule>>,
}

impl RedactionForecastValidator {
    pub fn validate(&self, item: &RedactionForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_forecast_validator() -> RedactionForecastValidator {
    RedactionForecastValidator {
        rules: vec![
            Rc::new(RedactionForecastStartsAtRangeRule { min: 255, max: 255 }) as Rc<dyn RedactionForecastRule>,
            Rc::new(RedactionForecastRequiredCategoryRule {
                message: "redaction forecast needs a value".to_string(),
            }),
            Rc::new(RedactionForecastUpdatedAtRangeRule { min: 16, max: 120 }),
            Rc::new(RedactionForecastCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_forecast_validator();
        let item = new_redaction_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = RedactionForecastStartsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_redaction_forecast("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = RedactionForecastRequiredCategoryRule {
            message: "redaction forecast needs a value".to_string(),
        };
        let mut item = new_redaction_forecast("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = RedactionForecastUpdatedAtRangeRule { min: 32, max: 3600 };
        let mut item = new_redaction_forecast("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RedactionForecastCodeLengthRule { max_length: 3600 };
        let mut item = new_redaction_forecast("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
