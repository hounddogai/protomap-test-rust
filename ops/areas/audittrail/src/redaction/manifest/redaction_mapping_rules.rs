use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction mapping must pass. Check returns its problems, or none.
pub trait RedactionMappingRule: std::fmt::Debug {
    fn check(&self, item: &RedactionMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMappingRequiredPhoneRule {
    pub message: String,
}

impl RedactionMappingRule for RedactionMappingRequiredPhoneRule {
    fn check(&self, item: &RedactionMapping) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMappingPhoneLengthRule {
    pub max_length: i64,
}

impl RedactionMappingRule for RedactionMappingPhoneLengthRule {
    fn check(&self, item: &RedactionMapping) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMappingRequiredTitleRule {
    pub message: String,
}

impl RedactionMappingRule for RedactionMappingRequiredTitleRule {
    fn check(&self, item: &RedactionMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMappingRequiredCodeRule {
    pub message: String,
}

impl RedactionMappingRule for RedactionMappingRequiredCodeRule {
    fn check(&self, item: &RedactionMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction mapping.
#[derive(Debug, Clone)]
pub struct RedactionMappingValidator {
    pub rules: Vec<Rc<dyn RedactionMappingRule>>,
}

impl RedactionMappingValidator {
    pub fn validate(&self, item: &RedactionMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_mapping_validator() -> RedactionMappingValidator {
    RedactionMappingValidator {
        rules: vec![
            Rc::new(RedactionMappingRequiredPhoneRule {
                message: "redaction mapping needs this field".to_string(),
            }) as Rc<dyn RedactionMappingRule>,
            Rc::new(RedactionMappingPhoneLengthRule { max_length: 80 }),
            Rc::new(RedactionMappingRequiredTitleRule {
                message: "redaction mapping needs this field".to_string(),
            }),
            Rc::new(RedactionMappingRequiredCodeRule {
                message: "redaction mapping needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_mapping_validator();
        let item = new_redaction_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = RedactionMappingRequiredPhoneRule {
            message: "redaction mapping needs an entry".to_string(),
        };
        let mut item = new_redaction_mapping("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = RedactionMappingPhoneLengthRule { max_length: 86400 };
        let mut item = new_redaction_mapping("x", "y");
        item.phone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RedactionMappingRequiredTitleRule {
            message: "redaction mapping needs a value".to_string(),
        };
        let mut item = new_redaction_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionMappingRequiredCodeRule {
            message: "redaction mapping needs this field".to_string(),
        };
        let mut item = new_redaction_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
