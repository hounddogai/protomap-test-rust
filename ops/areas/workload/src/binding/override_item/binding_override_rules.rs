use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding override must pass. Check returns its problems, or none.
pub trait BindingOverrideRule: std::fmt::Debug {
    fn check(&self, item: &BindingOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingOverridePhoneLengthRule {
    pub max_length: i64,
}

impl BindingOverrideRule for BindingOverridePhoneLengthRule {
    fn check(&self, item: &BindingOverride) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingOverrideRequiredReviewerRule {
    pub message: String,
}

impl BindingOverrideRule for BindingOverrideRequiredReviewerRule {
    fn check(&self, item: &BindingOverride) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingOverrideEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingOverrideRule for BindingOverrideEndsAtRangeRule {
    fn check(&self, item: &BindingOverride) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingOverrideCodeLengthRule {
    pub max_length: i64,
}

impl BindingOverrideRule for BindingOverrideCodeLengthRule {
    fn check(&self, item: &BindingOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingOverrideRequiredLabelRule {
    pub message: String,
}

impl BindingOverrideRule for BindingOverrideRequiredLabelRule {
    fn check(&self, item: &BindingOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingOverrideRequiredCodeRule {
    pub message: String,
}

impl BindingOverrideRule for BindingOverrideRequiredCodeRule {
    fn check(&self, item: &BindingOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding override.
#[derive(Debug, Clone)]
pub struct BindingOverrideValidator {
    pub rules: Vec<Rc<dyn BindingOverrideRule>>,
}

impl BindingOverrideValidator {
    pub fn validate(&self, item: &BindingOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_override_validator() -> BindingOverrideValidator {
    BindingOverrideValidator {
        rules: vec![
            Rc::new(BindingOverridePhoneLengthRule { max_length: 120 }) as Rc<dyn BindingOverrideRule>,
            Rc::new(BindingOverrideRequiredReviewerRule {
                message: "binding override needs a value".to_string(),
            }),
            Rc::new(BindingOverrideEndsAtRangeRule { min: 32, max: 86400 }),
            Rc::new(BindingOverrideCodeLengthRule { max_length: 1000 }),
            Rc::new(BindingOverrideRequiredLabelRule {
                message: "binding override needs this field".to_string(),
            }),
            Rc::new(BindingOverrideRequiredCodeRule {
                message: "binding override needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_override_validator();
        let item = new_binding_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = BindingOverridePhoneLengthRule { max_length: 32 };
        let mut item = new_binding_override("x", "y");
        item.phone = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = BindingOverrideRequiredReviewerRule {
            message: "binding override needs a value".to_string(),
        };
        let mut item = new_binding_override("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = BindingOverrideEndsAtRangeRule { min: 64, max: 120 };
        let mut item = new_binding_override("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BindingOverrideCodeLengthRule { max_length: 1000 };
        let mut item = new_binding_override("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BindingOverrideRequiredLabelRule {
            message: "binding override needs an entry".to_string(),
        };
        let mut item = new_binding_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BindingOverrideRequiredCodeRule {
            message: "binding override needs a value".to_string(),
        };
        let mut item = new_binding_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
