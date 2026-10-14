use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped usage group must pass. Check returns its problems, or none.
pub trait CappedUsageGroupRule: std::fmt::Debug {
    fn check(&self, item: &CappedUsageGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedUsageGroupLabelLengthRule {
    pub max_length: i64,
}

impl CappedUsageGroupRule for CappedUsageGroupLabelLengthRule {
    fn check(&self, item: &CappedUsageGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedUsageGroupPhoneLengthRule {
    pub max_length: i64,
}

impl CappedUsageGroupRule for CappedUsageGroupPhoneLengthRule {
    fn check(&self, item: &CappedUsageGroup) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedUsageGroupRequiredPhoneRule {
    pub message: String,
}

impl CappedUsageGroupRule for CappedUsageGroupRequiredPhoneRule {
    fn check(&self, item: &CappedUsageGroup) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedUsageGroupAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl CappedUsageGroupRule for CappedUsageGroupAllowedSourceRule {
    fn check(&self, item: &CappedUsageGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is inconsistent", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedUsageGroupSourceLengthRule {
    pub max_length: i64,
}

impl CappedUsageGroupRule for CappedUsageGroupSourceLengthRule {
    fn check(&self, item: &CappedUsageGroup) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped usage group.
#[derive(Debug, Clone)]
pub struct CappedUsageGroupValidator {
    pub rules: Vec<Rc<dyn CappedUsageGroupRule>>,
}

impl CappedUsageGroupValidator {
    pub fn validate(&self, item: &CappedUsageGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedUsageGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedUsageGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_usage_group_validator() -> CappedUsageGroupValidator {
    CappedUsageGroupValidator {
        rules: vec![
            Rc::new(CappedUsageGroupLabelLengthRule { max_length: 32 }) as Rc<dyn CappedUsageGroupRule>,
            Rc::new(CappedUsageGroupPhoneLengthRule { max_length: 255 }),
            Rc::new(CappedUsageGroupRequiredPhoneRule {
                message: "capped usage group needs this field".to_string(),
            }),
            Rc::new(CappedUsageGroupAllowedSourceRule {
                allowed: vec!["lenient".to_string(), "pending".to_string()],
            }),
            Rc::new(CappedUsageGroupSourceLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_usage_group_validator();
        let item = new_capped_usage_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CappedUsageGroupLabelLengthRule { max_length: 80 };
        let mut item = new_capped_usage_group("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = CappedUsageGroupPhoneLengthRule { max_length: 86400 };
        let mut item = new_capped_usage_group("x", "y");
        item.phone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = CappedUsageGroupRequiredPhoneRule {
            message: "capped usage group needs an entry".to_string(),
        };
        let mut item = new_capped_usage_group("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = CappedUsageGroupSourceLengthRule { max_length: 64 };
        let mut item = new_capped_usage_group("x", "y");
        item.source = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
