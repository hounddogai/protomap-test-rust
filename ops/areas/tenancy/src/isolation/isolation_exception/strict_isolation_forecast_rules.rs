use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict isolation forecast must pass. Check returns its problems, or none.
pub trait StrictIsolationForecastRule: std::fmt::Debug {
    fn check(&self, item: &StrictIsolationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationForecastRequiredLabelRule {
    pub message: String,
}

impl StrictIsolationForecastRule for StrictIsolationForecastRequiredLabelRule {
    fn check(&self, item: &StrictIsolationForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationForecastRequiredScopeRule {
    pub message: String,
}

impl StrictIsolationForecastRule for StrictIsolationForecastRequiredScopeRule {
    fn check(&self, item: &StrictIsolationForecast) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationForecastScopeLengthRule {
    pub max_length: i64,
}

impl StrictIsolationForecastRule for StrictIsolationForecastScopeLengthRule {
    fn check(&self, item: &StrictIsolationForecast) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationForecastRequiredTenantIdRule {
    pub message: String,
}

impl StrictIsolationForecastRule for StrictIsolationForecastRequiredTenantIdRule {
    fn check(&self, item: &StrictIsolationForecast) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationForecastRequiredCodeRule {
    pub message: String,
}

impl StrictIsolationForecastRule for StrictIsolationForecastRequiredCodeRule {
    fn check(&self, item: &StrictIsolationForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationForecastCodeLengthRule {
    pub max_length: i64,
}

impl StrictIsolationForecastRule for StrictIsolationForecastCodeLengthRule {
    fn check(&self, item: &StrictIsolationForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict isolation forecast.
#[derive(Debug, Clone)]
pub struct StrictIsolationForecastValidator {
    pub rules: Vec<Rc<dyn StrictIsolationForecastRule>>,
}

impl StrictIsolationForecastValidator {
    pub fn validate(&self, item: &StrictIsolationForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictIsolationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictIsolationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_isolation_forecast_validator() -> StrictIsolationForecastValidator {
    StrictIsolationForecastValidator {
        rules: vec![
            Rc::new(StrictIsolationForecastRequiredLabelRule {
                message: "strict isolation forecast needs an entry".to_string(),
            }) as Rc<dyn StrictIsolationForecastRule>,
            Rc::new(StrictIsolationForecastRequiredScopeRule {
                message: "strict isolation forecast needs a value".to_string(),
            }),
            Rc::new(StrictIsolationForecastScopeLengthRule { max_length: 32 }),
            Rc::new(StrictIsolationForecastRequiredTenantIdRule {
                message: "strict isolation forecast needs this field".to_string(),
            }),
            Rc::new(StrictIsolationForecastRequiredCodeRule {
                message: "strict isolation forecast needs a value".to_string(),
            }),
            Rc::new(StrictIsolationForecastCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_isolation_forecast_validator();
        let item = new_strict_isolation_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictIsolationForecastRequiredLabelRule {
            message: "strict isolation forecast needs an entry".to_string(),
        };
        let mut item = new_strict_isolation_forecast("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = StrictIsolationForecastRequiredScopeRule {
            message: "strict isolation forecast needs a value".to_string(),
        };
        let mut item = new_strict_isolation_forecast("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = StrictIsolationForecastScopeLengthRule { max_length: 32 };
        let mut item = new_strict_isolation_forecast("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = StrictIsolationForecastRequiredTenantIdRule {
            message: "strict isolation forecast needs an entry".to_string(),
        };
        let mut item = new_strict_isolation_forecast("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictIsolationForecastRequiredCodeRule {
            message: "strict isolation forecast needs an entry".to_string(),
        };
        let mut item = new_strict_isolation_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictIsolationForecastCodeLengthRule { max_length: 255 };
        let mut item = new_strict_isolation_forecast("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
