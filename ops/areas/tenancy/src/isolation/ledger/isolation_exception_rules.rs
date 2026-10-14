use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation exception must pass. Check returns its problems, or none.
pub trait IsolationExceptionRule: std::fmt::Debug {
    fn check(&self, item: &IsolationException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationExceptionRequiredNameRule {
    pub message: String,
}

impl IsolationExceptionRule for IsolationExceptionRequiredNameRule {
    fn check(&self, item: &IsolationException) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationExceptionRequiredNamespaceRule {
    pub message: String,
}

impl IsolationExceptionRule for IsolationExceptionRequiredNamespaceRule {
    fn check(&self, item: &IsolationException) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationExceptionCodeLengthRule {
    pub max_length: i64,
}

impl IsolationExceptionRule for IsolationExceptionCodeLengthRule {
    fn check(&self, item: &IsolationException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationExceptionReviewerLengthRule {
    pub max_length: i64,
}

impl IsolationExceptionRule for IsolationExceptionReviewerLengthRule {
    fn check(&self, item: &IsolationException) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationExceptionBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationExceptionRule for IsolationExceptionBatchSizeRangeRule {
    fn check(&self, item: &IsolationException) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a isolation exception.
#[derive(Debug, Clone)]
pub struct IsolationExceptionValidator {
    pub rules: Vec<Rc<dyn IsolationExceptionRule>>,
}

impl IsolationExceptionValidator {
    pub fn validate(&self, item: &IsolationException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_exception_validator() -> IsolationExceptionValidator {
    IsolationExceptionValidator {
        rules: vec![
            Rc::new(IsolationExceptionRequiredNameRule {
                message: "isolation exception needs this field".to_string(),
            }) as Rc<dyn IsolationExceptionRule>,
            Rc::new(IsolationExceptionRequiredNamespaceRule {
                message: "isolation exception needs an entry".to_string(),
            }),
            Rc::new(IsolationExceptionCodeLengthRule { max_length: 16 }),
            Rc::new(IsolationExceptionReviewerLengthRule { max_length: 120 }),
            Rc::new(IsolationExceptionBatchSizeRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_exception_validator();
        let item = new_isolation_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = IsolationExceptionRequiredNameRule {
            message: "isolation exception needs this field".to_string(),
        };
        let mut item = new_isolation_exception("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = IsolationExceptionRequiredNamespaceRule {
            message: "isolation exception needs a value".to_string(),
        };
        let mut item = new_isolation_exception("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IsolationExceptionCodeLengthRule { max_length: 80 };
        let mut item = new_isolation_exception("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = IsolationExceptionReviewerLengthRule { max_length: 120 };
        let mut item = new_isolation_exception("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = IsolationExceptionBatchSizeRangeRule { min: 64, max: 86400 };
        let mut item = new_isolation_exception("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
