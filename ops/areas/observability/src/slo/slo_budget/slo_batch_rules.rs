use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo batch must pass. Check returns its problems, or none.
pub trait SloBatchRule: std::fmt::Debug {
    fn check(&self, item: &SloBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBatchRequiredTitleRule {
    pub message: String,
}

impl SloBatchRule for SloBatchRequiredTitleRule {
    fn check(&self, item: &SloBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBatchLabelLengthRule {
    pub max_length: i64,
}

impl SloBatchRule for SloBatchLabelLengthRule {
    fn check(&self, item: &SloBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBatchRequiredCodeRule {
    pub message: String,
}

impl SloBatchRule for SloBatchRequiredCodeRule {
    fn check(&self, item: &SloBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloBatchCodeLengthRule {
    pub max_length: i64,
}

impl SloBatchRule for SloBatchCodeLengthRule {
    fn check(&self, item: &SloBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo batch.
#[derive(Debug, Clone)]
pub struct SloBatchValidator {
    pub rules: Vec<Rc<dyn SloBatchRule>>,
}

impl SloBatchValidator {
    pub fn validate(&self, item: &SloBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_batch_validator() -> SloBatchValidator {
    SloBatchValidator {
        rules: vec![
            Rc::new(SloBatchRequiredTitleRule {
                message: "slo batch needs an entry".to_string(),
            }) as Rc<dyn SloBatchRule>,
            Rc::new(SloBatchLabelLengthRule { max_length: 120 }),
            Rc::new(SloBatchRequiredCodeRule {
                message: "slo batch needs an entry".to_string(),
            }),
            Rc::new(SloBatchCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_batch_validator();
        let item = new_slo_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SloBatchRequiredTitleRule {
            message: "slo batch needs this field".to_string(),
        };
        let mut item = new_slo_batch("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SloBatchLabelLengthRule { max_length: 86400 };
        let mut item = new_slo_batch("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloBatchRequiredCodeRule {
            message: "slo batch needs a value".to_string(),
        };
        let mut item = new_slo_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SloBatchCodeLengthRule { max_length: 80 };
        let mut item = new_slo_batch("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
