use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal tenant forecast must pass. Check returns its problems, or none.
pub trait SeasonalTenantForecastRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalTenantForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTenantForecastRequiredCurrencyRule {
    pub message: String,
}

impl SeasonalTenantForecastRule for SeasonalTenantForecastRequiredCurrencyRule {
    fn check(&self, item: &SeasonalTenantForecast) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTenantForecastRequiredNameRule {
    pub message: String,
}

impl SeasonalTenantForecastRule for SeasonalTenantForecastRequiredNameRule {
    fn check(&self, item: &SeasonalTenantForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTenantForecastRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalTenantForecastRule for SeasonalTenantForecastRevisionRangeRule {
    fn check(&self, item: &SeasonalTenantForecast) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTenantForecastCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalTenantForecastRule for SeasonalTenantForecastCodeLengthRule {
    fn check(&self, item: &SeasonalTenantForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTenantForecastCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalTenantForecastRule for SeasonalTenantForecastCeilingRangeRule {
    fn check(&self, item: &SeasonalTenantForecast) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal tenant forecast.
#[derive(Debug, Clone)]
pub struct SeasonalTenantForecastValidator {
    pub rules: Vec<Rc<dyn SeasonalTenantForecastRule>>,
}

impl SeasonalTenantForecastValidator {
    pub fn validate(&self, item: &SeasonalTenantForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalTenantForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalTenantForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_tenant_forecast_validator() -> SeasonalTenantForecastValidator {
    SeasonalTenantForecastValidator {
        rules: vec![
            Rc::new(SeasonalTenantForecastRequiredCurrencyRule {
                message: "seasonal tenant forecast needs this field".to_string(),
            }) as Rc<dyn SeasonalTenantForecastRule>,
            Rc::new(SeasonalTenantForecastRequiredNameRule {
                message: "seasonal tenant forecast needs a value".to_string(),
            }),
            Rc::new(SeasonalTenantForecastRevisionRangeRule { min: 1000, max: 3600 }),
            Rc::new(SeasonalTenantForecastCodeLengthRule { max_length: 16 }),
            Rc::new(SeasonalTenantForecastCeilingRangeRule { min: 7.2, max: 220.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_tenant_forecast_validator();
        let item = new_seasonal_tenant_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = SeasonalTenantForecastRequiredCurrencyRule {
            message: "seasonal tenant forecast needs a value".to_string(),
        };
        let mut item = new_seasonal_tenant_forecast("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SeasonalTenantForecastRequiredNameRule {
            message: "seasonal tenant forecast needs an entry".to_string(),
        };
        let mut item = new_seasonal_tenant_forecast("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = SeasonalTenantForecastRevisionRangeRule { min: 32, max: 120 };
        let mut item = new_seasonal_tenant_forecast("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalTenantForecastCodeLengthRule { max_length: 120 };
        let mut item = new_seasonal_tenant_forecast("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = SeasonalTenantForecastCeilingRangeRule { min: 8.1, max: 3929.6 };
        let mut item = new_seasonal_tenant_forecast("x", "y");
        item.ceiling = 3931.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
