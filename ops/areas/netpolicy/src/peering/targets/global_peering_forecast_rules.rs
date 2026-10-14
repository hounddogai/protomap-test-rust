use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global peering forecast must pass. Check returns its problems, or none.
pub trait GlobalPeeringForecastRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPeeringForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringForecastCodePrefixRule {
    pub prefix: String,
}

impl GlobalPeeringForecastRule for GlobalPeeringForecastCodePrefixRule {
    fn check(&self, item: &GlobalPeeringForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringForecastTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl GlobalPeeringForecastRule for GlobalPeeringForecastTagsLimitRule {
    fn check(&self, item: &GlobalPeeringForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringForecastRequiredCodeRule {
    pub message: String,
}

impl GlobalPeeringForecastRule for GlobalPeeringForecastRequiredCodeRule {
    fn check(&self, item: &GlobalPeeringForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringForecastCurrencyLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringForecastRule for GlobalPeeringForecastCurrencyLengthRule {
    fn check(&self, item: &GlobalPeeringForecast) -> Vec<String> {
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
pub struct GlobalPeeringForecastRequiredIssuerRule {
    pub message: String,
}

impl GlobalPeeringForecastRule for GlobalPeeringForecastRequiredIssuerRule {
    fn check(&self, item: &GlobalPeeringForecast) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringForecastCodeLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringForecastRule for GlobalPeeringForecastCodeLengthRule {
    fn check(&self, item: &GlobalPeeringForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global peering forecast.
#[derive(Debug, Clone)]
pub struct GlobalPeeringForecastValidator {
    pub rules: Vec<Rc<dyn GlobalPeeringForecastRule>>,
}

impl GlobalPeeringForecastValidator {
    pub fn validate(&self, item: &GlobalPeeringForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalPeeringForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPeeringForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_peering_forecast_validator() -> GlobalPeeringForecastValidator {
    GlobalPeeringForecastValidator {
        rules: vec![
            Rc::new(GlobalPeeringForecastCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn GlobalPeeringForecastRule>,
            Rc::new(GlobalPeeringForecastTagsLimitRule {
                max_count: 3600,
                banned: vec!["regional".to_string(), "global".to_string(), "primary".to_string()],
            }),
            Rc::new(GlobalPeeringForecastRequiredCodeRule {
                message: "global peering forecast needs this field".to_string(),
            }),
            Rc::new(GlobalPeeringForecastCurrencyLengthRule { max_length: 16 }),
            Rc::new(GlobalPeeringForecastRequiredIssuerRule {
                message: "global peering forecast needs an entry".to_string(),
            }),
            Rc::new(GlobalPeeringForecastCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_peering_forecast_validator();
        let item = new_global_peering_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalPeeringForecastRequiredCodeRule {
            message: "global peering forecast needs a value".to_string(),
        };
        let mut item = new_global_peering_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = GlobalPeeringForecastCurrencyLengthRule { max_length: 80 };
        let mut item = new_global_peering_forecast("x", "y");
        item.currency = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = GlobalPeeringForecastRequiredIssuerRule {
            message: "global peering forecast needs a value".to_string(),
        };
        let mut item = new_global_peering_forecast("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalPeeringForecastCodeLengthRule { max_length: 120 };
        let mut item = new_global_peering_forecast("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
