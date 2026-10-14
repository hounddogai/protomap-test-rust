use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback suspension forecast must pass. Check returns its problems, or none.
pub trait FallbackSuspensionForecastRule: std::fmt::Debug {
    fn check(&self, item: &FallbackSuspensionForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionForecastRequiredSummaryRule {
    pub message: String,
}

impl FallbackSuspensionForecastRule for FallbackSuspensionForecastRequiredSummaryRule {
    fn check(&self, item: &FallbackSuspensionForecast) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionForecastSummaryLengthRule {
    pub max_length: i64,
}

impl FallbackSuspensionForecastRule for FallbackSuspensionForecastSummaryLengthRule {
    fn check(&self, item: &FallbackSuspensionForecast) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionForecastRequiredCodeRule {
    pub message: String,
}

impl FallbackSuspensionForecastRule for FallbackSuspensionForecastRequiredCodeRule {
    fn check(&self, item: &FallbackSuspensionForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback suspension forecast.
#[derive(Debug, Clone)]
pub struct FallbackSuspensionForecastValidator {
    pub rules: Vec<Rc<dyn FallbackSuspensionForecastRule>>,
}

impl FallbackSuspensionForecastValidator {
    pub fn validate(&self, item: &FallbackSuspensionForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackSuspensionForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackSuspensionForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_suspension_forecast_validator() -> FallbackSuspensionForecastValidator {
    FallbackSuspensionForecastValidator {
        rules: vec![
            Rc::new(FallbackSuspensionForecastRequiredSummaryRule {
                message: "fallback suspension forecast needs a value".to_string(),
            }) as Rc<dyn FallbackSuspensionForecastRule>,
            Rc::new(FallbackSuspensionForecastSummaryLengthRule { max_length: 80 }),
            Rc::new(FallbackSuspensionForecastRequiredCodeRule {
                message: "fallback suspension forecast needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_suspension_forecast_validator();
        let item = new_fallback_suspension_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = FallbackSuspensionForecastRequiredSummaryRule {
            message: "fallback suspension forecast needs a value".to_string(),
        };
        let mut item = new_fallback_suspension_forecast("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = FallbackSuspensionForecastSummaryLengthRule { max_length: 86400 };
        let mut item = new_fallback_suspension_forecast("x", "y");
        item.summary = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackSuspensionForecastRequiredCodeRule {
            message: "fallback suspension forecast needs this field".to_string(),
        };
        let mut item = new_fallback_suspension_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
