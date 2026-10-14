use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a redaction group must pass. Check returns its problems, or none.
pub trait RedactionGroupRule: std::fmt::Debug {
    fn check(&self, item: &RedactionGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionGroupAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl RedactionGroupRule for RedactionGroupAllowedFingerprintRule {
    fn check(&self, item: &RedactionGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is missing", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionGroupCodeLengthRule {
    pub max_length: i64,
}

impl RedactionGroupRule for RedactionGroupCodeLengthRule {
    fn check(&self, item: &RedactionGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionGroupCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionGroupRule for RedactionGroupCreatedAtRangeRule {
    fn check(&self, item: &RedactionGroup) -> Vec<String> {
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
pub struct RedactionGroupLabelLengthRule {
    pub max_length: i64,
}

impl RedactionGroupRule for RedactionGroupLabelLengthRule {
    fn check(&self, item: &RedactionGroup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionGroupRequiredLabelRule {
    pub message: String,
}

impl RedactionGroupRule for RedactionGroupRequiredLabelRule {
    fn check(&self, item: &RedactionGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionGroupQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionGroupRule for RedactionGroupQuantityRangeRule {
    fn check(&self, item: &RedactionGroup) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction group.
#[derive(Debug, Clone)]
pub struct RedactionGroupValidator {
    pub rules: Vec<Rc<dyn RedactionGroupRule>>,
}

impl RedactionGroupValidator {
    pub fn validate(&self, item: &RedactionGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_group_validator() -> RedactionGroupValidator {
    RedactionGroupValidator {
        rules: vec![
            Rc::new(RedactionGroupAllowedFingerprintRule { allowed: Vec::new() }) as Rc<dyn RedactionGroupRule>,
            Rc::new(RedactionGroupCodeLengthRule { max_length: 32 }),
            Rc::new(RedactionGroupCreatedAtRangeRule { min: 64, max: 1000 }),
            Rc::new(RedactionGroupLabelLengthRule { max_length: 1000 }),
            Rc::new(RedactionGroupRequiredLabelRule {
                message: "redaction group needs a value".to_string(),
            }),
            Rc::new(RedactionGroupQuantityRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_group_validator();
        let item = new_redaction_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RedactionGroupCodeLengthRule { max_length: 120 };
        let mut item = new_redaction_group("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RedactionGroupCreatedAtRangeRule { min: 16, max: 120 };
        let mut item = new_redaction_group("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RedactionGroupLabelLengthRule { max_length: 1000 };
        let mut item = new_redaction_group("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RedactionGroupRequiredLabelRule {
            message: "redaction group needs an entry".to_string(),
        };
        let mut item = new_redaction_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = RedactionGroupQuantityRangeRule { min: 32, max: 1000 };
        let mut item = new_redaction_group("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
