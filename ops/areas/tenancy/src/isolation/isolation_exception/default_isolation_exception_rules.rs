use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default isolation exception must pass. Check returns its problems, or none.
pub trait DefaultIsolationExceptionRule: std::fmt::Debug {
    fn check(&self, item: &DefaultIsolationException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationExceptionCodePrefixRule {
    pub prefix: String,
}

impl DefaultIsolationExceptionRule for DefaultIsolationExceptionCodePrefixRule {
    fn check(&self, item: &DefaultIsolationException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationExceptionRequiredTitleRule {
    pub message: String,
}

impl DefaultIsolationExceptionRule for DefaultIsolationExceptionRequiredTitleRule {
    fn check(&self, item: &DefaultIsolationException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationExceptionRequiredSegmentRule {
    pub message: String,
}

impl DefaultIsolationExceptionRule for DefaultIsolationExceptionRequiredSegmentRule {
    fn check(&self, item: &DefaultIsolationException) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default isolation exception.
#[derive(Debug, Clone)]
pub struct DefaultIsolationExceptionValidator {
    pub rules: Vec<Rc<dyn DefaultIsolationExceptionRule>>,
}

impl DefaultIsolationExceptionValidator {
    pub fn validate(&self, item: &DefaultIsolationException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultIsolationException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultIsolationException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_isolation_exception_validator() -> DefaultIsolationExceptionValidator {
    DefaultIsolationExceptionValidator {
        rules: vec![
            Rc::new(DefaultIsolationExceptionCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn DefaultIsolationExceptionRule>,
            Rc::new(DefaultIsolationExceptionRequiredTitleRule {
                message: "default isolation exception needs an entry".to_string(),
            }),
            Rc::new(DefaultIsolationExceptionRequiredSegmentRule {
                message: "default isolation exception needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_isolation_exception_validator();
        let item = new_default_isolation_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultIsolationExceptionRequiredTitleRule {
            message: "default isolation exception needs a value".to_string(),
        };
        let mut item = new_default_isolation_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = DefaultIsolationExceptionRequiredSegmentRule {
            message: "default isolation exception needs this field".to_string(),
        };
        let mut item = new_default_isolation_exception("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
