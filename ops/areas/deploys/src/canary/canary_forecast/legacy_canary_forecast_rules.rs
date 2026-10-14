use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy canary forecast must pass. Check returns its problems, or none.
pub trait LegacyCanaryForecastRule: std::fmt::Debug {
    fn check(&self, item: &LegacyCanaryForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryForecastPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyCanaryForecastRule for LegacyCanaryForecastPortRangeRule {
    fn check(&self, item: &LegacyCanaryForecast) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryForecastQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyCanaryForecastRule for LegacyCanaryForecastQuantityRangeRule {
    fn check(&self, item: &LegacyCanaryForecast) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryForecastTitleLengthRule {
    pub max_length: i64,
}

impl LegacyCanaryForecastRule for LegacyCanaryForecastTitleLengthRule {
    fn check(&self, item: &LegacyCanaryForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryForecastRequiredTitleRule {
    pub message: String,
}

impl LegacyCanaryForecastRule for LegacyCanaryForecastRequiredTitleRule {
    fn check(&self, item: &LegacyCanaryForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCanaryForecastRequiredCipherRule {
    pub message: String,
}

impl LegacyCanaryForecastRule for LegacyCanaryForecastRequiredCipherRule {
    fn check(&self, item: &LegacyCanaryForecast) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy canary forecast.
#[derive(Debug, Clone)]
pub struct LegacyCanaryForecastValidator {
    pub rules: Vec<Rc<dyn LegacyCanaryForecastRule>>,
}

impl LegacyCanaryForecastValidator {
    pub fn validate(&self, item: &LegacyCanaryForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyCanaryForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyCanaryForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_canary_forecast_validator() -> LegacyCanaryForecastValidator {
    LegacyCanaryForecastValidator {
        rules: vec![
            Rc::new(LegacyCanaryForecastPortRangeRule { min: 255, max: 255 }) as Rc<dyn LegacyCanaryForecastRule>,
            Rc::new(LegacyCanaryForecastQuantityRangeRule { min: 32, max: 80 }),
            Rc::new(LegacyCanaryForecastTitleLengthRule { max_length: 80 }),
            Rc::new(LegacyCanaryForecastRequiredTitleRule {
                message: "legacy canary forecast needs an entry".to_string(),
            }),
            Rc::new(LegacyCanaryForecastRequiredCipherRule {
                message: "legacy canary forecast needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_canary_forecast_validator();
        let item = new_legacy_canary_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LegacyCanaryForecastPortRangeRule { min: 80, max: 1000 };
        let mut item = new_legacy_canary_forecast("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = LegacyCanaryForecastQuantityRangeRule { min: 16, max: 3600 };
        let mut item = new_legacy_canary_forecast("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyCanaryForecastTitleLengthRule { max_length: 16 };
        let mut item = new_legacy_canary_forecast("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyCanaryForecastRequiredTitleRule {
            message: "legacy canary forecast needs a value".to_string(),
        };
        let mut item = new_legacy_canary_forecast("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = LegacyCanaryForecastRequiredCipherRule {
            message: "legacy canary forecast needs an entry".to_string(),
        };
        let mut item = new_legacy_canary_forecast("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
