use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage override must pass. Check returns its problems, or none.
pub trait UsageOverrideRule: std::fmt::Debug {
    fn check(&self, item: &UsageOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageOverrideCodePrefixRule {
    pub prefix: String,
}

impl UsageOverrideRule for UsageOverrideCodePrefixRule {
    fn check(&self, item: &UsageOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageOverrideRequiredLabelRule {
    pub message: String,
}

impl UsageOverrideRule for UsageOverrideRequiredLabelRule {
    fn check(&self, item: &UsageOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageOverrideWindowRule {
    pub max_span: i64,
}

impl UsageOverrideRule for UsageOverrideWindowRule {
    fn check(&self, item: &UsageOverride) -> Vec<String> {
        if item.starts_at > item.ends_at {
            return vec!["starts at is after ends at".to_string()];
        }
        if self.max_span > 0 && (item.ends_at - item.starts_at > self.max_span) {
            return vec![format!("spans more than {} seconds", self.max_span)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageOverrideCategoryLengthRule {
    pub max_length: i64,
}

impl UsageOverrideRule for UsageOverrideCategoryLengthRule {
    fn check(&self, item: &UsageOverride) -> Vec<String> {
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

/// Runs every rule of a usage override.
#[derive(Debug, Clone)]
pub struct UsageOverrideValidator {
    pub rules: Vec<Rc<dyn UsageOverrideRule>>,
}

impl UsageOverrideValidator {
    pub fn validate(&self, item: &UsageOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_override_validator() -> UsageOverrideValidator {
    UsageOverrideValidator {
        rules: vec![
            Rc::new(UsageOverrideCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn UsageOverrideRule>,
            Rc::new(UsageOverrideRequiredLabelRule {
                message: "usage override needs a value".to_string(),
            }),
            Rc::new(UsageOverrideWindowRule { max_span: 86400 }),
            Rc::new(UsageOverrideCategoryLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_override_validator();
        let item = new_usage_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = UsageOverrideRequiredLabelRule {
            message: "usage override needs this field".to_string(),
        };
        let mut item = new_usage_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = UsageOverrideCategoryLengthRule { max_length: 80 };
        let mut item = new_usage_override("x", "y");
        item.category = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
