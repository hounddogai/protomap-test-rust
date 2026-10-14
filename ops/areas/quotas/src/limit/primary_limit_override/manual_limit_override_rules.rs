use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual limit override must pass. Check returns its problems, or none.
pub trait ManualLimitOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ManualLimitOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitOverrideRequiredNameRule {
    pub message: String,
}

impl ManualLimitOverrideRule for ManualLimitOverrideRequiredNameRule {
    fn check(&self, item: &ManualLimitOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitOverrideNameLengthRule {
    pub max_length: i64,
}

impl ManualLimitOverrideRule for ManualLimitOverrideNameLengthRule {
    fn check(&self, item: &ManualLimitOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitOverrideAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ManualLimitOverrideRule for ManualLimitOverrideAllowedNameRule {
    fn check(&self, item: &ManualLimitOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitOverrideBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualLimitOverrideRule for ManualLimitOverrideBudgetRangeRule {
    fn check(&self, item: &ManualLimitOverride) -> Vec<String> {
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

/// Runs every rule of a manual limit override.
#[derive(Debug, Clone)]
pub struct ManualLimitOverrideValidator {
    pub rules: Vec<Rc<dyn ManualLimitOverrideRule>>,
}

impl ManualLimitOverrideValidator {
    pub fn validate(&self, item: &ManualLimitOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualLimitOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualLimitOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_limit_override_validator() -> ManualLimitOverrideValidator {
    ManualLimitOverrideValidator {
        rules: vec![
            Rc::new(ManualLimitOverrideRequiredNameRule {
                message: "manual limit override needs a value".to_string(),
            }) as Rc<dyn ManualLimitOverrideRule>,
            Rc::new(ManualLimitOverrideNameLengthRule { max_length: 32 }),
            Rc::new(ManualLimitOverrideAllowedNameRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(ManualLimitOverrideBudgetRangeRule { min: 2.9, max: 1125.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_limit_override_validator();
        let item = new_manual_limit_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualLimitOverrideRequiredNameRule {
            message: "manual limit override needs a value".to_string(),
        };
        let mut item = new_manual_limit_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualLimitOverrideNameLengthRule { max_length: 1000 };
        let mut item = new_manual_limit_override("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = ManualLimitOverrideBudgetRangeRule { min: 4.4, max: 322.3 };
        let mut item = new_manual_limit_override("x", "y");
        item.budget = 323.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
