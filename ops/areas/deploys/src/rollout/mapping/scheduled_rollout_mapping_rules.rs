use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled rollout mapping must pass. Check returns its problems, or none.
pub trait ScheduledRolloutMappingRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRolloutMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutMappingRequiredCipherRule {
    pub message: String,
}

impl ScheduledRolloutMappingRule for ScheduledRolloutMappingRequiredCipherRule {
    fn check(&self, item: &ScheduledRolloutMapping) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutMappingCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledRolloutMappingRule for ScheduledRolloutMappingCodeLengthRule {
    fn check(&self, item: &ScheduledRolloutMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutMappingRequiredTitleRule {
    pub message: String,
}

impl ScheduledRolloutMappingRule for ScheduledRolloutMappingRequiredTitleRule {
    fn check(&self, item: &ScheduledRolloutMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled rollout mapping.
#[derive(Debug, Clone)]
pub struct ScheduledRolloutMappingValidator {
    pub rules: Vec<Rc<dyn ScheduledRolloutMappingRule>>,
}

impl ScheduledRolloutMappingValidator {
    pub fn validate(&self, item: &ScheduledRolloutMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledRolloutMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRolloutMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_rollout_mapping_validator() -> ScheduledRolloutMappingValidator {
    ScheduledRolloutMappingValidator {
        rules: vec![
            Rc::new(ScheduledRolloutMappingRequiredCipherRule {
                message: "scheduled rollout mapping needs a value".to_string(),
            }) as Rc<dyn ScheduledRolloutMappingRule>,
            Rc::new(ScheduledRolloutMappingCodeLengthRule { max_length: 1000 }),
            Rc::new(ScheduledRolloutMappingRequiredTitleRule {
                message: "scheduled rollout mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_rollout_mapping_validator();
        let item = new_scheduled_rollout_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = ScheduledRolloutMappingRequiredCipherRule {
            message: "scheduled rollout mapping needs this field".to_string(),
        };
        let mut item = new_scheduled_rollout_mapping("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledRolloutMappingCodeLengthRule { max_length: 64 };
        let mut item = new_scheduled_rollout_mapping("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledRolloutMappingRequiredTitleRule {
            message: "scheduled rollout mapping needs this field".to_string(),
        };
        let mut item = new_scheduled_rollout_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
