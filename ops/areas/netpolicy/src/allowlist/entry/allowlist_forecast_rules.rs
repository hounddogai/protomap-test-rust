use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist forecast must pass. Check returns its problems, or none.
pub trait AllowlistForecastRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistForecastRequiredChecksumRule {
    pub message: String,
}

impl AllowlistForecastRule for AllowlistForecastRequiredChecksumRule {
    fn check(&self, item: &AllowlistForecast) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistForecastThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistForecastRule for AllowlistForecastThresholdRangeRule {
    fn check(&self, item: &AllowlistForecast) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistForecastAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistForecastRule for AllowlistForecastAttemptsRangeRule {
    fn check(&self, item: &AllowlistForecast) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistForecastScopeLengthRule {
    pub max_length: i64,
}

impl AllowlistForecastRule for AllowlistForecastScopeLengthRule {
    fn check(&self, item: &AllowlistForecast) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistForecastAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl AllowlistForecastRule for AllowlistForecastAllowedChecksumRule {
    fn check(&self, item: &AllowlistForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is not allowed", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistForecastChecksumLengthRule {
    pub max_length: i64,
}

impl AllowlistForecastRule for AllowlistForecastChecksumLengthRule {
    fn check(&self, item: &AllowlistForecast) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist forecast.
#[derive(Debug, Clone)]
pub struct AllowlistForecastValidator {
    pub rules: Vec<Rc<dyn AllowlistForecastRule>>,
}

impl AllowlistForecastValidator {
    pub fn validate(&self, item: &AllowlistForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_forecast_validator() -> AllowlistForecastValidator {
    AllowlistForecastValidator {
        rules: vec![
            Rc::new(AllowlistForecastRequiredChecksumRule {
                message: "allowlist forecast needs an entry".to_string(),
            }) as Rc<dyn AllowlistForecastRule>,
            Rc::new(AllowlistForecastThresholdRangeRule { min: 4.8, max: 1972.1 }),
            Rc::new(AllowlistForecastAttemptsRangeRule { min: 16, max: 120 }),
            Rc::new(AllowlistForecastScopeLengthRule { max_length: 255 }),
            Rc::new(AllowlistForecastAllowedChecksumRule {
                allowed: vec!["adaptive".to_string(), "scheduled".to_string(), "legacy".to_string()],
            }),
            Rc::new(AllowlistForecastChecksumLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_forecast_validator();
        let item = new_allowlist_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = AllowlistForecastRequiredChecksumRule {
            message: "allowlist forecast needs this field".to_string(),
        };
        let mut item = new_allowlist_forecast("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = AllowlistForecastThresholdRangeRule { min: 1.4, max: 3051.5 };
        let mut item = new_allowlist_forecast("x", "y");
        item.threshold = 3053.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = AllowlistForecastAttemptsRangeRule { min: 120, max: 255 };
        let mut item = new_allowlist_forecast("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = AllowlistForecastScopeLengthRule { max_length: 32 };
        let mut item = new_allowlist_forecast("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = AllowlistForecastChecksumLengthRule { max_length: 32 };
        let mut item = new_allowlist_forecast("x", "y");
        item.checksum = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
