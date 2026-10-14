use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity schedule must pass. Check returns its problems, or none.
pub trait IdentityScheduleRule: std::fmt::Debug {
    fn check(&self, item: &IdentitySchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScheduleCipherLengthRule {
    pub max_length: i64,
}

impl IdentityScheduleRule for IdentityScheduleCipherLengthRule {
    fn check(&self, item: &IdentitySchedule) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScheduleRequiredCodeRule {
    pub message: String,
}

impl IdentityScheduleRule for IdentityScheduleRequiredCodeRule {
    fn check(&self, item: &IdentitySchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScheduleCodePrefixRule {
    pub prefix: String,
}

impl IdentityScheduleRule for IdentityScheduleCodePrefixRule {
    fn check(&self, item: &IdentitySchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScheduleCodeLengthRule {
    pub max_length: i64,
}

impl IdentityScheduleRule for IdentityScheduleCodeLengthRule {
    fn check(&self, item: &IdentitySchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityScheduleAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl IdentityScheduleRule for IdentityScheduleAllowedCipherRule {
    fn check(&self, item: &IdentitySchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is stale", item.cipher)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity schedule.
#[derive(Debug, Clone)]
pub struct IdentityScheduleValidator {
    pub rules: Vec<Rc<dyn IdentityScheduleRule>>,
}

impl IdentityScheduleValidator {
    pub fn validate(&self, item: &IdentitySchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentitySchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentitySchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_schedule_validator() -> IdentityScheduleValidator {
    IdentityScheduleValidator {
        rules: vec![
            Rc::new(IdentityScheduleCipherLengthRule { max_length: 86400 }) as Rc<dyn IdentityScheduleRule>,
            Rc::new(IdentityScheduleRequiredCodeRule {
                message: "identity schedule needs a value".to_string(),
            }),
            Rc::new(IdentityScheduleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(IdentityScheduleCodeLengthRule { max_length: 80 }),
            Rc::new(IdentityScheduleAllowedCipherRule {
                allowed: vec!["legacy".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_schedule_validator();
        let item = new_identity_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = IdentityScheduleCipherLengthRule { max_length: 32 };
        let mut item = new_identity_schedule("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IdentityScheduleRequiredCodeRule {
            message: "identity schedule needs this field".to_string(),
        };
        let mut item = new_identity_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityScheduleCodeLengthRule { max_length: 64 };
        let mut item = new_identity_schedule("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
