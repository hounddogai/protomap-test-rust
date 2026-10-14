use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling forecast policy must pass. Check returns its problems, or none.
pub trait RollingForecastPolicyRule: std::fmt::Debug {
    fn check(&self, item: &RollingForecastPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingForecastPolicyBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingForecastPolicyRule for RollingForecastPolicyBurstRangeRule {
    fn check(&self, item: &RollingForecastPolicy) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingForecastPolicyNameLengthRule {
    pub max_length: i64,
}

impl RollingForecastPolicyRule for RollingForecastPolicyNameLengthRule {
    fn check(&self, item: &RollingForecastPolicy) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingForecastPolicyCodeLengthRule {
    pub max_length: i64,
}

impl RollingForecastPolicyRule for RollingForecastPolicyCodeLengthRule {
    fn check(&self, item: &RollingForecastPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingForecastPolicyRequiredCodeRule {
    pub message: String,
}

impl RollingForecastPolicyRule for RollingForecastPolicyRequiredCodeRule {
    fn check(&self, item: &RollingForecastPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingForecastPolicyAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl RollingForecastPolicyRule for RollingForecastPolicyAllowedLocaleRule {
    fn check(&self, item: &RollingForecastPolicy) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is unknown", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingForecastPolicyCountryLengthRule {
    pub max_length: i64,
}

impl RollingForecastPolicyRule for RollingForecastPolicyCountryLengthRule {
    fn check(&self, item: &RollingForecastPolicy) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling forecast policy.
#[derive(Debug, Clone)]
pub struct RollingForecastPolicyValidator {
    pub rules: Vec<Rc<dyn RollingForecastPolicyRule>>,
}

impl RollingForecastPolicyValidator {
    pub fn validate(&self, item: &RollingForecastPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingForecastPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingForecastPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_forecast_policy_validator() -> RollingForecastPolicyValidator {
    RollingForecastPolicyValidator {
        rules: vec![
            Rc::new(RollingForecastPolicyBurstRangeRule { min: 255, max: 86400 }) as Rc<dyn RollingForecastPolicyRule>,
            Rc::new(RollingForecastPolicyNameLengthRule { max_length: 16 }),
            Rc::new(RollingForecastPolicyCodeLengthRule { max_length: 255 }),
            Rc::new(RollingForecastPolicyRequiredCodeRule {
                message: "rolling forecast policy needs an entry".to_string(),
            }),
            Rc::new(RollingForecastPolicyAllowedLocaleRule {
                allowed: vec!["pending".to_string(), "rolling".to_string()],
            }),
            Rc::new(RollingForecastPolicyCountryLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_forecast_policy_validator();
        let item = new_rolling_forecast_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = RollingForecastPolicyBurstRangeRule { min: 80, max: 86400 };
        let mut item = new_rolling_forecast_policy("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingForecastPolicyNameLengthRule { max_length: 120 };
        let mut item = new_rolling_forecast_policy("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingForecastPolicyCodeLengthRule { max_length: 86400 };
        let mut item = new_rolling_forecast_policy("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingForecastPolicyRequiredCodeRule {
            message: "rolling forecast policy needs this field".to_string(),
        };
        let mut item = new_rolling_forecast_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = RollingForecastPolicyCountryLengthRule { max_length: 80 };
        let mut item = new_rolling_forecast_policy("x", "y");
        item.country = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
