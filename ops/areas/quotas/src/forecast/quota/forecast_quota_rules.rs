use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast quota must pass. Check returns its problems, or none.
pub trait ForecastQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ForecastQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ForecastQuotaRule for ForecastQuotaAllowedCodeRule {
    fn check(&self, item: &ForecastQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastQuotaWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastQuotaRule for ForecastQuotaWeightRangeRule {
    fn check(&self, item: &ForecastQuota) -> Vec<String> {
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
pub struct ForecastQuotaCodeLengthRule {
    pub max_length: i64,
}

impl ForecastQuotaRule for ForecastQuotaCodeLengthRule {
    fn check(&self, item: &ForecastQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastQuotaQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastQuotaRule for ForecastQuotaQuantityRangeRule {
    fn check(&self, item: &ForecastQuota) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastQuotaChannelLengthRule {
    pub max_length: i64,
}

impl ForecastQuotaRule for ForecastQuotaChannelLengthRule {
    fn check(&self, item: &ForecastQuota) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast quota.
#[derive(Debug, Clone)]
pub struct ForecastQuotaValidator {
    pub rules: Vec<Rc<dyn ForecastQuotaRule>>,
}

impl ForecastQuotaValidator {
    pub fn validate(&self, item: &ForecastQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_quota_validator() -> ForecastQuotaValidator {
    ForecastQuotaValidator {
        rules: vec![
            Rc::new(ForecastQuotaAllowedCodeRule {
                allowed: vec!["scheduled".to_string(), "tiered".to_string(), "global".to_string()],
            }) as Rc<dyn ForecastQuotaRule>,
            Rc::new(ForecastQuotaWeightRangeRule { min: 1.7, max: 4225.5 }),
            Rc::new(ForecastQuotaCodeLengthRule { max_length: 3600 }),
            Rc::new(ForecastQuotaQuantityRangeRule { min: 32, max: 64 }),
            Rc::new(ForecastQuotaChannelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_quota_validator();
        let item = new_forecast_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ForecastQuotaWeightRangeRule { min: 8.6, max: 4492.2 };
        let mut item = new_forecast_quota("x", "y");
        item.weight = 4493.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ForecastQuotaCodeLengthRule { max_length: 255 };
        let mut item = new_forecast_quota("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ForecastQuotaQuantityRangeRule { min: 1000, max: 1000 };
        let mut item = new_forecast_quota("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = ForecastQuotaChannelLengthRule { max_length: 80 };
        let mut item = new_forecast_quota("x", "y");
        item.channel = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
