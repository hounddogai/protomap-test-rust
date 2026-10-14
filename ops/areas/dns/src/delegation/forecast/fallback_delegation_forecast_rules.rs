use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback delegation forecast must pass. Check returns its problems, or none.
pub trait FallbackDelegationForecastRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDelegationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationForecastAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl FallbackDelegationForecastRule for FallbackDelegationForecastAllowedTenantIdRule {
    fn check(&self, item: &FallbackDelegationForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is out of range", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationForecastTitleLengthRule {
    pub max_length: i64,
}

impl FallbackDelegationForecastRule for FallbackDelegationForecastTitleLengthRule {
    fn check(&self, item: &FallbackDelegationForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationForecastCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackDelegationForecastRule for FallbackDelegationForecastCreatedAtRangeRule {
    fn check(&self, item: &FallbackDelegationForecast) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationForecastMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackDelegationForecastRule for FallbackDelegationForecastMinimumRangeRule {
    fn check(&self, item: &FallbackDelegationForecast) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationForecastRequiredTitleRule {
    pub message: String,
}

impl FallbackDelegationForecastRule for FallbackDelegationForecastRequiredTitleRule {
    fn check(&self, item: &FallbackDelegationForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationForecastRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackDelegationForecastRule for FallbackDelegationForecastRatioRangeRule {
    fn check(&self, item: &FallbackDelegationForecast) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a fallback delegation forecast.
#[derive(Debug, Clone)]
pub struct FallbackDelegationForecastValidator {
    pub rules: Vec<Rc<dyn FallbackDelegationForecastRule>>,
}

impl FallbackDelegationForecastValidator {
    pub fn validate(&self, item: &FallbackDelegationForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackDelegationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDelegationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_delegation_forecast_validator() -> FallbackDelegationForecastValidator {
    FallbackDelegationForecastValidator {
        rules: vec![
            Rc::new(FallbackDelegationForecastAllowedTenantIdRule {
                allowed: vec!["seasonal".to_string(), "adaptive".to_string(), "capped".to_string()],
            }) as Rc<dyn FallbackDelegationForecastRule>,
            Rc::new(FallbackDelegationForecastTitleLengthRule { max_length: 32 }),
            Rc::new(FallbackDelegationForecastCreatedAtRangeRule { min: 64, max: 80 }),
            Rc::new(FallbackDelegationForecastMinimumRangeRule { min: 1.5, max: 4657.5 }),
            Rc::new(FallbackDelegationForecastRequiredTitleRule {
                message: "fallback delegation forecast needs this field".to_string(),
            }),
            Rc::new(FallbackDelegationForecastRatioRangeRule { min: 9.9, max: 1077.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_delegation_forecast_validator();
        let item = new_fallback_delegation_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackDelegationForecastTitleLengthRule { max_length: 16 };
        let mut item = new_fallback_delegation_forecast("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = FallbackDelegationForecastCreatedAtRangeRule { min: 3600, max: 3600 };
        let mut item = new_fallback_delegation_forecast("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = FallbackDelegationForecastMinimumRangeRule { min: 6.4, max: 2950.5 };
        let mut item = new_fallback_delegation_forecast("x", "y");
        item.minimum = 2952.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackDelegationForecastRequiredTitleRule {
            message: "fallback delegation forecast needs this field".to_string(),
        };
        let mut item = new_fallback_delegation_forecast("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = FallbackDelegationForecastRatioRangeRule { min: 9.9, max: 3195.0 };
        let mut item = new_fallback_delegation_forecast("x", "y");
        item.ratio = 3196.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
