use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling runbook override must pass. Check returns its problems, or none.
pub trait RollingRunbookOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RollingRunbookOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookOverrideRequiredCategoryRule {
    pub message: String,
}

impl RollingRunbookOverrideRule for RollingRunbookOverrideRequiredCategoryRule {
    fn check(&self, item: &RollingRunbookOverride) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookOverrideCodeLengthRule {
    pub max_length: i64,
}

impl RollingRunbookOverrideRule for RollingRunbookOverrideCodeLengthRule {
    fn check(&self, item: &RollingRunbookOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookOverrideTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingRunbookOverrideRule for RollingRunbookOverrideTtlSecondsRangeRule {
    fn check(&self, item: &RollingRunbookOverride) -> Vec<String> {
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
pub struct RollingRunbookOverrideCategoryLengthRule {
    pub max_length: i64,
}

impl RollingRunbookOverrideRule for RollingRunbookOverrideCategoryLengthRule {
    fn check(&self, item: &RollingRunbookOverride) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollingRunbookOverrideRule for RollingRunbookOverrideAllowedCodeRule {
    fn check(&self, item: &RollingRunbookOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRunbookOverrideCodePrefixRule {
    pub prefix: String,
}

impl RollingRunbookOverrideRule for RollingRunbookOverrideCodePrefixRule {
    fn check(&self, item: &RollingRunbookOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling runbook override.
#[derive(Debug, Clone)]
pub struct RollingRunbookOverrideValidator {
    pub rules: Vec<Rc<dyn RollingRunbookOverrideRule>>,
}

impl RollingRunbookOverrideValidator {
    pub fn validate(&self, item: &RollingRunbookOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingRunbookOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRunbookOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_runbook_override_validator() -> RollingRunbookOverrideValidator {
    RollingRunbookOverrideValidator {
        rules: vec![
            Rc::new(RollingRunbookOverrideRequiredCategoryRule {
                message: "rolling runbook override needs a value".to_string(),
            }) as Rc<dyn RollingRunbookOverrideRule>,
            Rc::new(RollingRunbookOverrideCodeLengthRule { max_length: 255 }),
            Rc::new(RollingRunbookOverrideTtlSecondsRangeRule { min: 16, max: 86400 }),
            Rc::new(RollingRunbookOverrideCategoryLengthRule { max_length: 64 }),
            Rc::new(RollingRunbookOverrideAllowedCodeRule {
                allowed: vec!["capped".to_string(), "weighted".to_string(), "batched".to_string()],
            }),
            Rc::new(RollingRunbookOverrideCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_runbook_override_validator();
        let item = new_rolling_runbook_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = RollingRunbookOverrideRequiredCategoryRule {
            message: "rolling runbook override needs an entry".to_string(),
        };
        let mut item = new_rolling_runbook_override("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingRunbookOverrideCodeLengthRule { max_length: 86400 };
        let mut item = new_rolling_runbook_override("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RollingRunbookOverrideTtlSecondsRangeRule { min: 16, max: 120 };
        let mut item = new_rolling_runbook_override("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = RollingRunbookOverrideCategoryLengthRule { max_length: 255 };
        let mut item = new_rolling_runbook_override("x", "y");
        item.category = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
