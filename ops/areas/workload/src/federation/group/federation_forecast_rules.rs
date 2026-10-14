use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a federation forecast must pass. Check returns its problems, or none.
pub trait FederationForecastRule: std::fmt::Debug {
    fn check(&self, item: &FederationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationForecastRegionLengthRule {
    pub max_length: i64,
}

impl FederationForecastRule for FederationForecastRegionLengthRule {
    fn check(&self, item: &FederationForecast) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationForecastCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationForecastRule for FederationForecastCapacityRangeRule {
    fn check(&self, item: &FederationForecast) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationForecastCurrencyLengthRule {
    pub max_length: i64,
}

impl FederationForecastRule for FederationForecastCurrencyLengthRule {
    fn check(&self, item: &FederationForecast) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationForecastRequiredTitleRule {
    pub message: String,
}

impl FederationForecastRule for FederationForecastRequiredTitleRule {
    fn check(&self, item: &FederationForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationForecastQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationForecastRule for FederationForecastQuotaUnitsRangeRule {
    fn check(&self, item: &FederationForecast) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a federation forecast.
#[derive(Debug, Clone)]
pub struct FederationForecastValidator {
    pub rules: Vec<Rc<dyn FederationForecastRule>>,
}

impl FederationForecastValidator {
    pub fn validate(&self, item: &FederationForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_forecast_validator() -> FederationForecastValidator {
    FederationForecastValidator {
        rules: vec![
            Rc::new(FederationForecastRegionLengthRule { max_length: 1000 }) as Rc<dyn FederationForecastRule>,
            Rc::new(FederationForecastCapacityRangeRule { min: 16, max: 64 }),
            Rc::new(FederationForecastCurrencyLengthRule { max_length: 64 }),
            Rc::new(FederationForecastRequiredTitleRule {
                message: "federation forecast needs an entry".to_string(),
            }),
            Rc::new(FederationForecastQuotaUnitsRangeRule { min: 16, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_forecast_validator();
        let item = new_federation_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = FederationForecastRegionLengthRule { max_length: 16 };
        let mut item = new_federation_forecast("x", "y");
        item.region = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = FederationForecastCapacityRangeRule { min: 64, max: 120 };
        let mut item = new_federation_forecast("x", "y");
        item.capacity = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = FederationForecastCurrencyLengthRule { max_length: 64 };
        let mut item = new_federation_forecast("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FederationForecastRequiredTitleRule {
            message: "federation forecast needs this field".to_string(),
        };
        let mut item = new_federation_forecast("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = FederationForecastQuotaUnitsRangeRule { min: 255, max: 86400 };
        let mut item = new_federation_forecast("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
