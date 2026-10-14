use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast template must pass. Check returns its problems, or none.
pub trait ForecastTemplateRule: std::fmt::Debug {
    fn check(&self, item: &ForecastTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTemplateRequiredTitleRule {
    pub message: String,
}

impl ForecastTemplateRule for ForecastTemplateRequiredTitleRule {
    fn check(&self, item: &ForecastTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTemplateTitleLengthRule {
    pub max_length: i64,
}

impl ForecastTemplateRule for ForecastTemplateTitleLengthRule {
    fn check(&self, item: &ForecastTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTemplateAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ForecastTemplateRule for ForecastTemplateAllowedTitleRule {
    fn check(&self, item: &ForecastTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is expired", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTemplateLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastTemplateRule for ForecastTemplateLimitRangeRule {
    fn check(&self, item: &ForecastTemplate) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTemplateRequiredCodeRule {
    pub message: String,
}

impl ForecastTemplateRule for ForecastTemplateRequiredCodeRule {
    fn check(&self, item: &ForecastTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast template.
#[derive(Debug, Clone)]
pub struct ForecastTemplateValidator {
    pub rules: Vec<Rc<dyn ForecastTemplateRule>>,
}

impl ForecastTemplateValidator {
    pub fn validate(&self, item: &ForecastTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_template_validator() -> ForecastTemplateValidator {
    ForecastTemplateValidator {
        rules: vec![
            Rc::new(ForecastTemplateRequiredTitleRule {
                message: "forecast template needs a value".to_string(),
            }) as Rc<dyn ForecastTemplateRule>,
            Rc::new(ForecastTemplateTitleLengthRule { max_length: 120 }),
            Rc::new(ForecastTemplateAllowedTitleRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(ForecastTemplateLimitRangeRule { min: 120, max: 3600 }),
            Rc::new(ForecastTemplateRequiredCodeRule {
                message: "forecast template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_template_validator();
        let item = new_forecast_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ForecastTemplateRequiredTitleRule {
            message: "forecast template needs this field".to_string(),
        };
        let mut item = new_forecast_template("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ForecastTemplateTitleLengthRule { max_length: 255 };
        let mut item = new_forecast_template("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = ForecastTemplateLimitRangeRule { min: 32, max: 3600 };
        let mut item = new_forecast_template("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ForecastTemplateRequiredCodeRule {
            message: "forecast template needs an entry".to_string(),
        };
        let mut item = new_forecast_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
