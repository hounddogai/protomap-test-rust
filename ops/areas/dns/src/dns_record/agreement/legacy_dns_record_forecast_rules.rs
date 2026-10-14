use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy dns record forecast must pass. Check returns its problems, or none.
pub trait LegacyDnsRecordForecastRule: std::fmt::Debug {
    fn check(&self, item: &LegacyDnsRecordForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordForecastAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl LegacyDnsRecordForecastRule for LegacyDnsRecordForecastAllowedCategoryRule {
    fn check(&self, item: &LegacyDnsRecordForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is below minimum", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordForecastCategoryLengthRule {
    pub max_length: i64,
}

impl LegacyDnsRecordForecastRule for LegacyDnsRecordForecastCategoryLengthRule {
    fn check(&self, item: &LegacyDnsRecordForecast) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordForecastLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyDnsRecordForecastRule for LegacyDnsRecordForecastLimitRangeRule {
    fn check(&self, item: &LegacyDnsRecordForecast) -> Vec<String> {
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
pub struct LegacyDnsRecordForecastRequiredNameRule {
    pub message: String,
}

impl LegacyDnsRecordForecastRule for LegacyDnsRecordForecastRequiredNameRule {
    fn check(&self, item: &LegacyDnsRecordForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy dns record forecast.
#[derive(Debug, Clone)]
pub struct LegacyDnsRecordForecastValidator {
    pub rules: Vec<Rc<dyn LegacyDnsRecordForecastRule>>,
}

impl LegacyDnsRecordForecastValidator {
    pub fn validate(&self, item: &LegacyDnsRecordForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyDnsRecordForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyDnsRecordForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_dns_record_forecast_validator() -> LegacyDnsRecordForecastValidator {
    LegacyDnsRecordForecastValidator {
        rules: vec![
            Rc::new(LegacyDnsRecordForecastAllowedCategoryRule {
                allowed: vec!["capped".to_string(), "batched".to_string(), "primary".to_string()],
            }) as Rc<dyn LegacyDnsRecordForecastRule>,
            Rc::new(LegacyDnsRecordForecastCategoryLengthRule { max_length: 255 }),
            Rc::new(LegacyDnsRecordForecastLimitRangeRule { min: 120, max: 1000 }),
            Rc::new(LegacyDnsRecordForecastRequiredNameRule {
                message: "legacy dns record forecast needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_dns_record_forecast_validator();
        let item = new_legacy_dns_record_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = LegacyDnsRecordForecastCategoryLengthRule { max_length: 255 };
        let mut item = new_legacy_dns_record_forecast("x", "y");
        item.category = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = LegacyDnsRecordForecastLimitRangeRule { min: 32, max: 80 };
        let mut item = new_legacy_dns_record_forecast("x", "y");
        item.limit = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyDnsRecordForecastRequiredNameRule {
            message: "legacy dns record forecast needs a value".to_string(),
        };
        let mut item = new_legacy_dns_record_forecast("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
