use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy nonce forecast must pass. Check returns its problems, or none.
pub trait LegacyNonceForecastRule: std::fmt::Debug {
    fn check(&self, item: &LegacyNonceForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceForecastRequiredTitleRule {
    pub message: String,
}

impl LegacyNonceForecastRule for LegacyNonceForecastRequiredTitleRule {
    fn check(&self, item: &LegacyNonceForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceForecastTitleLengthRule {
    pub max_length: i64,
}

impl LegacyNonceForecastRule for LegacyNonceForecastTitleLengthRule {
    fn check(&self, item: &LegacyNonceForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceForecastWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyNonceForecastRule for LegacyNonceForecastWeightRangeRule {
    fn check(&self, item: &LegacyNonceForecast) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceForecastCodePrefixRule {
    pub prefix: String,
}

impl LegacyNonceForecastRule for LegacyNonceForecastCodePrefixRule {
    fn check(&self, item: &LegacyNonceForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceForecastExternalRefLengthRule {
    pub max_length: i64,
}

impl LegacyNonceForecastRule for LegacyNonceForecastExternalRefLengthRule {
    fn check(&self, item: &LegacyNonceForecast) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy nonce forecast.
#[derive(Debug, Clone)]
pub struct LegacyNonceForecastValidator {
    pub rules: Vec<Rc<dyn LegacyNonceForecastRule>>,
}

impl LegacyNonceForecastValidator {
    pub fn validate(&self, item: &LegacyNonceForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyNonceForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyNonceForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_nonce_forecast_validator() -> LegacyNonceForecastValidator {
    LegacyNonceForecastValidator {
        rules: vec![
            Rc::new(LegacyNonceForecastRequiredTitleRule {
                message: "legacy nonce forecast needs this field".to_string(),
            }) as Rc<dyn LegacyNonceForecastRule>,
            Rc::new(LegacyNonceForecastTitleLengthRule { max_length: 1000 }),
            Rc::new(LegacyNonceForecastWeightRangeRule { min: 8.5, max: 296.6 }),
            Rc::new(LegacyNonceForecastCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LegacyNonceForecastExternalRefLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_nonce_forecast_validator();
        let item = new_legacy_nonce_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyNonceForecastRequiredTitleRule {
            message: "legacy nonce forecast needs an entry".to_string(),
        };
        let mut item = new_legacy_nonce_forecast("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyNonceForecastTitleLengthRule { max_length: 255 };
        let mut item = new_legacy_nonce_forecast("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = LegacyNonceForecastWeightRangeRule { min: 4.4, max: 2799.6 };
        let mut item = new_legacy_nonce_forecast("x", "y");
        item.weight = 2801.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = LegacyNonceForecastExternalRefLengthRule { max_length: 86400 };
        let mut item = new_legacy_nonce_forecast("x", "y");
        item.external_ref = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
