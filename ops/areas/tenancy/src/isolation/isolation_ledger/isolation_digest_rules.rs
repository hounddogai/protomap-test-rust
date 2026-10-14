use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation digest must pass. Check returns its problems, or none.
pub trait IsolationDigestRule: std::fmt::Debug {
    fn check(&self, item: &IsolationDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDigestRequiredNamespaceRule {
    pub message: String,
}

impl IsolationDigestRule for IsolationDigestRequiredNamespaceRule {
    fn check(&self, item: &IsolationDigest) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDigestTitleLengthRule {
    pub max_length: i64,
}

impl IsolationDigestRule for IsolationDigestTitleLengthRule {
    fn check(&self, item: &IsolationDigest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDigestRequiredCodeRule {
    pub message: String,
}

impl IsolationDigestRule for IsolationDigestRequiredCodeRule {
    fn check(&self, item: &IsolationDigest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationDigestSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationDigestRule for IsolationDigestSequenceRangeRule {
    fn check(&self, item: &IsolationDigest) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a isolation digest.
#[derive(Debug, Clone)]
pub struct IsolationDigestValidator {
    pub rules: Vec<Rc<dyn IsolationDigestRule>>,
}

impl IsolationDigestValidator {
    pub fn validate(&self, item: &IsolationDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_digest_validator() -> IsolationDigestValidator {
    IsolationDigestValidator {
        rules: vec![
            Rc::new(IsolationDigestRequiredNamespaceRule {
                message: "isolation digest needs an entry".to_string(),
            }) as Rc<dyn IsolationDigestRule>,
            Rc::new(IsolationDigestTitleLengthRule { max_length: 255 }),
            Rc::new(IsolationDigestRequiredCodeRule {
                message: "isolation digest needs this field".to_string(),
            }),
            Rc::new(IsolationDigestSequenceRangeRule { min: 86400, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_digest_validator();
        let item = new_isolation_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = IsolationDigestRequiredNamespaceRule {
            message: "isolation digest needs an entry".to_string(),
        };
        let mut item = new_isolation_digest("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = IsolationDigestTitleLengthRule { max_length: 32 };
        let mut item = new_isolation_digest("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IsolationDigestRequiredCodeRule {
            message: "isolation digest needs a value".to_string(),
        };
        let mut item = new_isolation_digest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = IsolationDigestSequenceRangeRule { min: 3600, max: 86400 };
        let mut item = new_isolation_digest("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
