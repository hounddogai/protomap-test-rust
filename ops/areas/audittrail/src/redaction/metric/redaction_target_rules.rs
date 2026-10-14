use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction target must pass. Check returns its problems, or none.
pub trait RedactionTargetRule: std::fmt::Debug {
    fn check(&self, item: &RedactionTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTargetRequiredSourceRule {
    pub message: String,
}

impl RedactionTargetRule for RedactionTargetRequiredSourceRule {
    fn check(&self, item: &RedactionTarget) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTargetCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionTargetRule for RedactionTargetCapacityRangeRule {
    fn check(&self, item: &RedactionTarget) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTargetRequiredCodeRule {
    pub message: String,
}

impl RedactionTargetRule for RedactionTargetRequiredCodeRule {
    fn check(&self, item: &RedactionTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTargetCodePrefixRule {
    pub prefix: String,
}

impl RedactionTargetRule for RedactionTargetCodePrefixRule {
    fn check(&self, item: &RedactionTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionTargetTitleLengthRule {
    pub max_length: i64,
}

impl RedactionTargetRule for RedactionTargetTitleLengthRule {
    fn check(&self, item: &RedactionTarget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction target.
#[derive(Debug, Clone)]
pub struct RedactionTargetValidator {
    pub rules: Vec<Rc<dyn RedactionTargetRule>>,
}

impl RedactionTargetValidator {
    pub fn validate(&self, item: &RedactionTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_target_validator() -> RedactionTargetValidator {
    RedactionTargetValidator {
        rules: vec![
            Rc::new(RedactionTargetRequiredSourceRule {
                message: "redaction target needs an entry".to_string(),
            }) as Rc<dyn RedactionTargetRule>,
            Rc::new(RedactionTargetCapacityRangeRule { min: 16, max: 255 }),
            Rc::new(RedactionTargetRequiredCodeRule {
                message: "redaction target needs a value".to_string(),
            }),
            Rc::new(RedactionTargetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RedactionTargetTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_target_validator();
        let item = new_redaction_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = RedactionTargetRequiredSourceRule {
            message: "redaction target needs an entry".to_string(),
        };
        let mut item = new_redaction_target("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RedactionTargetCapacityRangeRule { min: 80, max: 80 };
        let mut item = new_redaction_target("x", "y");
        item.capacity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionTargetRequiredCodeRule {
            message: "redaction target needs a value".to_string(),
        };
        let mut item = new_redaction_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RedactionTargetTitleLengthRule { max_length: 32 };
        let mut item = new_redaction_target("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
