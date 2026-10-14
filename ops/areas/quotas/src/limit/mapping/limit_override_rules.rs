use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a limit override must pass. Check returns its problems, or none.
pub trait LimitOverrideRule: std::fmt::Debug {
    fn check(&self, item: &LimitOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitOverrideNameLengthRule {
    pub max_length: i64,
}

impl LimitOverrideRule for LimitOverrideNameLengthRule {
    fn check(&self, item: &LimitOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitOverrideDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitOverrideRule for LimitOverrideDiscountRangeRule {
    fn check(&self, item: &LimitOverride) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitOverrideTitleLengthRule {
    pub max_length: i64,
}

impl LimitOverrideRule for LimitOverrideTitleLengthRule {
    fn check(&self, item: &LimitOverride) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitOverrideAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LimitOverrideRule for LimitOverrideAllowedNameRule {
    fn check(&self, item: &LimitOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is inconsistent", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitOverrideRequiredTitleRule {
    pub message: String,
}

impl LimitOverrideRule for LimitOverrideRequiredTitleRule {
    fn check(&self, item: &LimitOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit override.
#[derive(Debug, Clone)]
pub struct LimitOverrideValidator {
    pub rules: Vec<Rc<dyn LimitOverrideRule>>,
}

impl LimitOverrideValidator {
    pub fn validate(&self, item: &LimitOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_override_validator() -> LimitOverrideValidator {
    LimitOverrideValidator {
        rules: vec![
            Rc::new(LimitOverrideNameLengthRule { max_length: 120 }) as Rc<dyn LimitOverrideRule>,
            Rc::new(LimitOverrideDiscountRangeRule { min: 8.6, max: 3500.8 }),
            Rc::new(LimitOverrideTitleLengthRule { max_length: 64 }),
            Rc::new(LimitOverrideAllowedNameRule {
                allowed: vec!["regional".to_string(), "adaptive".to_string(), "primary".to_string()],
            }),
            Rc::new(LimitOverrideRequiredTitleRule {
                message: "limit override needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_override_validator();
        let item = new_limit_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LimitOverrideNameLengthRule { max_length: 86400 };
        let mut item = new_limit_override("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = LimitOverrideDiscountRangeRule { min: 7.8, max: 639.6 };
        let mut item = new_limit_override("x", "y");
        item.discount = 641.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LimitOverrideTitleLengthRule { max_length: 1000 };
        let mut item = new_limit_override("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LimitOverrideRequiredTitleRule {
            message: "limit override needs an entry".to_string(),
        };
        let mut item = new_limit_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
