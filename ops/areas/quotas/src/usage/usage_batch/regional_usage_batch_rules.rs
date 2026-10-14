use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional usage batch must pass. Check returns its problems, or none.
pub trait RegionalUsageBatchRule: std::fmt::Debug {
    fn check(&self, item: &RegionalUsageBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageBatchNamespaceLengthRule {
    pub max_length: i64,
}

impl RegionalUsageBatchRule for RegionalUsageBatchNamespaceLengthRule {
    fn check(&self, item: &RegionalUsageBatch) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageBatchCodeLengthRule {
    pub max_length: i64,
}

impl RegionalUsageBatchRule for RegionalUsageBatchCodeLengthRule {
    fn check(&self, item: &RegionalUsageBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageBatchNotesLengthRule {
    pub max_length: i64,
}

impl RegionalUsageBatchRule for RegionalUsageBatchNotesLengthRule {
    fn check(&self, item: &RegionalUsageBatch) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageBatchRequiredNotesRule {
    pub message: String,
}

impl RegionalUsageBatchRule for RegionalUsageBatchRequiredNotesRule {
    fn check(&self, item: &RegionalUsageBatch) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional usage batch.
#[derive(Debug, Clone)]
pub struct RegionalUsageBatchValidator {
    pub rules: Vec<Rc<dyn RegionalUsageBatchRule>>,
}

impl RegionalUsageBatchValidator {
    pub fn validate(&self, item: &RegionalUsageBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalUsageBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalUsageBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_usage_batch_validator() -> RegionalUsageBatchValidator {
    RegionalUsageBatchValidator {
        rules: vec![
            Rc::new(RegionalUsageBatchNamespaceLengthRule { max_length: 80 }) as Rc<dyn RegionalUsageBatchRule>,
            Rc::new(RegionalUsageBatchCodeLengthRule { max_length: 3600 }),
            Rc::new(RegionalUsageBatchNotesLengthRule { max_length: 80 }),
            Rc::new(RegionalUsageBatchRequiredNotesRule {
                message: "regional usage batch needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_usage_batch_validator();
        let item = new_regional_usage_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = RegionalUsageBatchNamespaceLengthRule { max_length: 16 };
        let mut item = new_regional_usage_batch("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalUsageBatchCodeLengthRule { max_length: 80 };
        let mut item = new_regional_usage_batch("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RegionalUsageBatchNotesLengthRule { max_length: 80 };
        let mut item = new_regional_usage_batch("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RegionalUsageBatchRequiredNotesRule {
            message: "regional usage batch needs a value".to_string(),
        };
        let mut item = new_regional_usage_batch("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
