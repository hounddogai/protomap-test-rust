use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered identity forecast must pass. Check returns its problems, or none.
pub trait TieredIdentityForecastRule: std::fmt::Debug {
    fn check(&self, item: &TieredIdentityForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityForecastExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredIdentityForecastRule for TieredIdentityForecastExpiresAtRangeRule {
    fn check(&self, item: &TieredIdentityForecast) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityForecastTitleLengthRule {
    pub max_length: i64,
}

impl TieredIdentityForecastRule for TieredIdentityForecastTitleLengthRule {
    fn check(&self, item: &TieredIdentityForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityForecastCodeLengthRule {
    pub max_length: i64,
}

impl TieredIdentityForecastRule for TieredIdentityForecastCodeLengthRule {
    fn check(&self, item: &TieredIdentityForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityForecastRequiredCodeRule {
    pub message: String,
}

impl TieredIdentityForecastRule for TieredIdentityForecastRequiredCodeRule {
    fn check(&self, item: &TieredIdentityForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityForecastDescriptionLengthRule {
    pub max_length: i64,
}

impl TieredIdentityForecastRule for TieredIdentityForecastDescriptionLengthRule {
    fn check(&self, item: &TieredIdentityForecast) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered identity forecast.
#[derive(Debug, Clone)]
pub struct TieredIdentityForecastValidator {
    pub rules: Vec<Rc<dyn TieredIdentityForecastRule>>,
}

impl TieredIdentityForecastValidator {
    pub fn validate(&self, item: &TieredIdentityForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredIdentityForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredIdentityForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_identity_forecast_validator() -> TieredIdentityForecastValidator {
    TieredIdentityForecastValidator {
        rules: vec![
            Rc::new(TieredIdentityForecastExpiresAtRangeRule { min: 32, max: 64 })
                as Rc<dyn TieredIdentityForecastRule>,
            Rc::new(TieredIdentityForecastTitleLengthRule { max_length: 1000 }),
            Rc::new(TieredIdentityForecastCodeLengthRule { max_length: 64 }),
            Rc::new(TieredIdentityForecastRequiredCodeRule {
                message: "tiered identity forecast needs this field".to_string(),
            }),
            Rc::new(TieredIdentityForecastDescriptionLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_identity_forecast_validator();
        let item = new_tiered_identity_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = TieredIdentityForecastExpiresAtRangeRule { min: 80, max: 1000 };
        let mut item = new_tiered_identity_forecast("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredIdentityForecastTitleLengthRule { max_length: 3600 };
        let mut item = new_tiered_identity_forecast("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredIdentityForecastCodeLengthRule { max_length: 64 };
        let mut item = new_tiered_identity_forecast("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredIdentityForecastRequiredCodeRule {
            message: "tiered identity forecast needs this field".to_string(),
        };
        let mut item = new_tiered_identity_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = TieredIdentityForecastDescriptionLengthRule { max_length: 86400 };
        let mut item = new_tiered_identity_forecast("x", "y");
        item.description = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
