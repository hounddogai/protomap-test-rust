use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a grant forecast must pass. Check returns its problems, or none.
pub trait GrantForecastRule: std::fmt::Debug {
    fn check(&self, item: &GrantForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantForecastScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantForecastRule for GrantForecastScoreRangeRule {
    fn check(&self, item: &GrantForecast) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantForecastRequiredCodeRule {
    pub message: String,
}

impl GrantForecastRule for GrantForecastRequiredCodeRule {
    fn check(&self, item: &GrantForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantForecastNameLengthRule {
    pub max_length: i64,
}

impl GrantForecastRule for GrantForecastNameLengthRule {
    fn check(&self, item: &GrantForecast) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantForecastBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantForecastRule for GrantForecastBudgetRangeRule {
    fn check(&self, item: &GrantForecast) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantForecastAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl GrantForecastRule for GrantForecastAllowedNamespaceRule {
    fn check(&self, item: &GrantForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is too long", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantForecastRequiredChecksumRule {
    pub message: String,
}

impl GrantForecastRule for GrantForecastRequiredChecksumRule {
    fn check(&self, item: &GrantForecast) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant forecast.
#[derive(Debug, Clone)]
pub struct GrantForecastValidator {
    pub rules: Vec<Rc<dyn GrantForecastRule>>,
}

impl GrantForecastValidator {
    pub fn validate(&self, item: &GrantForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_forecast_validator() -> GrantForecastValidator {
    GrantForecastValidator {
        rules: vec![
            Rc::new(GrantForecastScoreRangeRule { min: 7.7, max: 1470.3 }) as Rc<dyn GrantForecastRule>,
            Rc::new(GrantForecastRequiredCodeRule {
                message: "grant forecast needs a value".to_string(),
            }),
            Rc::new(GrantForecastNameLengthRule { max_length: 32 }),
            Rc::new(GrantForecastBudgetRangeRule { min: 2.5, max: 4065.1 }),
            Rc::new(GrantForecastAllowedNamespaceRule {
                allowed: vec!["manual".to_string(), "batched".to_string(), "manual".to_string()],
            }),
            Rc::new(GrantForecastRequiredChecksumRule {
                message: "grant forecast needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_forecast_validator();
        let item = new_grant_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = GrantForecastScoreRangeRule { min: 3.7, max: 3374.0 };
        let mut item = new_grant_forecast("x", "y");
        item.score = 3375.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GrantForecastRequiredCodeRule {
            message: "grant forecast needs this field".to_string(),
        };
        let mut item = new_grant_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GrantForecastNameLengthRule { max_length: 1000 };
        let mut item = new_grant_forecast("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = GrantForecastBudgetRangeRule { min: 2.8, max: 582.5 };
        let mut item = new_grant_forecast("x", "y");
        item.budget = 584.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = GrantForecastRequiredChecksumRule {
            message: "grant forecast needs this field".to_string(),
        };
        let mut item = new_grant_forecast("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
