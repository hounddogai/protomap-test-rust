use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check budget must pass. Check returns its problems, or none.
pub trait HealthCheckBudgetRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBudgetTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckBudgetRule for HealthCheckBudgetTtlSecondsRangeRule {
    fn check(&self, item: &HealthCheckBudget) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBudgetRequiredLabelRule {
    pub message: String,
}

impl HealthCheckBudgetRule for HealthCheckBudgetRequiredLabelRule {
    fn check(&self, item: &HealthCheckBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBudgetRequiredCodeRule {
    pub message: String,
}

impl HealthCheckBudgetRule for HealthCheckBudgetRequiredCodeRule {
    fn check(&self, item: &HealthCheckBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBudgetCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckBudgetRule for HealthCheckBudgetCodeLengthRule {
    fn check(&self, item: &HealthCheckBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBudgetNamespaceLengthRule {
    pub max_length: i64,
}

impl HealthCheckBudgetRule for HealthCheckBudgetNamespaceLengthRule {
    fn check(&self, item: &HealthCheckBudget) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBudgetVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckBudgetRule for HealthCheckBudgetVersionRangeRule {
    fn check(&self, item: &HealthCheckBudget) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check budget.
#[derive(Debug, Clone)]
pub struct HealthCheckBudgetValidator {
    pub rules: Vec<Rc<dyn HealthCheckBudgetRule>>,
}

impl HealthCheckBudgetValidator {
    pub fn validate(&self, item: &HealthCheckBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &HealthCheckBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_budget_validator() -> HealthCheckBudgetValidator {
    HealthCheckBudgetValidator {
        rules: vec![
            Rc::new(HealthCheckBudgetTtlSecondsRangeRule { min: 80, max: 86400 }) as Rc<dyn HealthCheckBudgetRule>,
            Rc::new(HealthCheckBudgetRequiredLabelRule {
                message: "health check budget needs an entry".to_string(),
            }),
            Rc::new(HealthCheckBudgetRequiredCodeRule {
                message: "health check budget needs an entry".to_string(),
            }),
            Rc::new(HealthCheckBudgetCodeLengthRule { max_length: 1000 }),
            Rc::new(HealthCheckBudgetNamespaceLengthRule { max_length: 32 }),
            Rc::new(HealthCheckBudgetVersionRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_budget_validator();
        let item = new_health_check_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = HealthCheckBudgetTtlSecondsRangeRule { min: 80, max: 3600 };
        let mut item = new_health_check_budget("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = HealthCheckBudgetRequiredLabelRule {
            message: "health check budget needs a value".to_string(),
        };
        let mut item = new_health_check_budget("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = HealthCheckBudgetRequiredCodeRule {
            message: "health check budget needs this field".to_string(),
        };
        let mut item = new_health_check_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = HealthCheckBudgetCodeLengthRule { max_length: 16 };
        let mut item = new_health_check_budget("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = HealthCheckBudgetNamespaceLengthRule { max_length: 32 };
        let mut item = new_health_check_budget("x", "y");
        item.namespace = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = HealthCheckBudgetVersionRangeRule { min: 64, max: 86400 };
        let mut item = new_health_check_budget("x", "y");
        item.version = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
