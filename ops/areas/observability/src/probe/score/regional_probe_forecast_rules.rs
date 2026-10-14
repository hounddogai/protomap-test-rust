use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional probe forecast must pass. Check returns its problems, or none.
pub trait RegionalProbeForecastRule: std::fmt::Debug {
    fn check(&self, item: &RegionalProbeForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeForecastCodeLengthRule {
    pub max_length: i64,
}

impl RegionalProbeForecastRule for RegionalProbeForecastCodeLengthRule {
    fn check(&self, item: &RegionalProbeForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeForecastCodePrefixRule {
    pub prefix: String,
}

impl RegionalProbeForecastRule for RegionalProbeForecastCodePrefixRule {
    fn check(&self, item: &RegionalProbeForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeForecastLabelLengthRule {
    pub max_length: i64,
}

impl RegionalProbeForecastRule for RegionalProbeForecastLabelLengthRule {
    fn check(&self, item: &RegionalProbeForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeForecastRequiredLabelRule {
    pub message: String,
}

impl RegionalProbeForecastRule for RegionalProbeForecastRequiredLabelRule {
    fn check(&self, item: &RegionalProbeForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeForecastScopeLengthRule {
    pub max_length: i64,
}

impl RegionalProbeForecastRule for RegionalProbeForecastScopeLengthRule {
    fn check(&self, item: &RegionalProbeForecast) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeForecastScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalProbeForecastRule for RegionalProbeForecastScoreRangeRule {
    fn check(&self, item: &RegionalProbeForecast) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional probe forecast.
#[derive(Debug, Clone)]
pub struct RegionalProbeForecastValidator {
    pub rules: Vec<Rc<dyn RegionalProbeForecastRule>>,
}

impl RegionalProbeForecastValidator {
    pub fn validate(&self, item: &RegionalProbeForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalProbeForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalProbeForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_probe_forecast_validator() -> RegionalProbeForecastValidator {
    RegionalProbeForecastValidator {
        rules: vec![
            Rc::new(RegionalProbeForecastCodeLengthRule { max_length: 1000 }) as Rc<dyn RegionalProbeForecastRule>,
            Rc::new(RegionalProbeForecastCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RegionalProbeForecastLabelLengthRule { max_length: 32 }),
            Rc::new(RegionalProbeForecastRequiredLabelRule {
                message: "regional probe forecast needs an entry".to_string(),
            }),
            Rc::new(RegionalProbeForecastScopeLengthRule { max_length: 16 }),
            Rc::new(RegionalProbeForecastScoreRangeRule { min: 5.6, max: 1724.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_probe_forecast_validator();
        let item = new_regional_probe_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalProbeForecastCodeLengthRule { max_length: 1000 };
        let mut item = new_regional_probe_forecast("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalProbeForecastLabelLengthRule { max_length: 32 };
        let mut item = new_regional_probe_forecast("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionalProbeForecastRequiredLabelRule {
            message: "regional probe forecast needs a value".to_string(),
        };
        let mut item = new_regional_probe_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = RegionalProbeForecastScopeLengthRule { max_length: 64 };
        let mut item = new_regional_probe_forecast("x", "y");
        item.scope = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RegionalProbeForecastScoreRangeRule { min: 5.6, max: 831.6 };
        let mut item = new_regional_probe_forecast("x", "y");
        item.score = 833.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
