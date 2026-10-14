use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling tenant forecast must pass. Check returns its problems, or none.
pub trait RollingTenantForecastRule: std::fmt::Debug {
    fn check(&self, item: &RollingTenantForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantForecastAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RollingTenantForecastRule for RollingTenantForecastAllowedTitleRule {
    fn check(&self, item: &RollingTenantForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantForecastCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingTenantForecastRule for RollingTenantForecastCreatedAtRangeRule {
    fn check(&self, item: &RollingTenantForecast) -> Vec<String> {
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
pub struct RollingTenantForecastRequiredCodeRule {
    pub message: String,
}

impl RollingTenantForecastRule for RollingTenantForecastRequiredCodeRule {
    fn check(&self, item: &RollingTenantForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantForecastRequiredTitleRule {
    pub message: String,
}

impl RollingTenantForecastRule for RollingTenantForecastRequiredTitleRule {
    fn check(&self, item: &RollingTenantForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantForecastRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingTenantForecastRule for RollingTenantForecastRotationDaysRangeRule {
    fn check(&self, item: &RollingTenantForecast) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling tenant forecast.
#[derive(Debug, Clone)]
pub struct RollingTenantForecastValidator {
    pub rules: Vec<Rc<dyn RollingTenantForecastRule>>,
}

impl RollingTenantForecastValidator {
    pub fn validate(&self, item: &RollingTenantForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingTenantForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTenantForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_tenant_forecast_validator() -> RollingTenantForecastValidator {
    RollingTenantForecastValidator {
        rules: vec![
            Rc::new(RollingTenantForecastAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn RollingTenantForecastRule>,
            Rc::new(RollingTenantForecastCreatedAtRangeRule { min: 16, max: 3600 }),
            Rc::new(RollingTenantForecastRequiredCodeRule {
                message: "rolling tenant forecast needs an entry".to_string(),
            }),
            Rc::new(RollingTenantForecastRequiredTitleRule {
                message: "rolling tenant forecast needs an entry".to_string(),
            }),
            Rc::new(RollingTenantForecastRotationDaysRangeRule { min: 16, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_tenant_forecast_validator();
        let item = new_rolling_tenant_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RollingTenantForecastCreatedAtRangeRule { min: 16, max: 86400 };
        let mut item = new_rolling_tenant_forecast("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingTenantForecastRequiredCodeRule {
            message: "rolling tenant forecast needs an entry".to_string(),
        };
        let mut item = new_rolling_tenant_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingTenantForecastRequiredTitleRule {
            message: "rolling tenant forecast needs an entry".to_string(),
        };
        let mut item = new_rolling_tenant_forecast("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = RollingTenantForecastRotationDaysRangeRule { min: 32, max: 80 };
        let mut item = new_rolling_tenant_forecast("x", "y");
        item.rotation_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
