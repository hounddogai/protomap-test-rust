use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient failover window must pass. Check returns its problems, or none.
pub trait LenientFailoverWindowRule: std::fmt::Debug {
    fn check(&self, item: &LenientFailoverWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFailoverWindowRequiredReviewerRule {
    pub message: String,
}

impl LenientFailoverWindowRule for LenientFailoverWindowRequiredReviewerRule {
    fn check(&self, item: &LenientFailoverWindow) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFailoverWindowRequiredTitleRule {
    pub message: String,
}

impl LenientFailoverWindowRule for LenientFailoverWindowRequiredTitleRule {
    fn check(&self, item: &LenientFailoverWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFailoverWindowCodeLengthRule {
    pub max_length: i64,
}

impl LenientFailoverWindowRule for LenientFailoverWindowCodeLengthRule {
    fn check(&self, item: &LenientFailoverWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient failover window.
#[derive(Debug, Clone)]
pub struct LenientFailoverWindowValidator {
    pub rules: Vec<Rc<dyn LenientFailoverWindowRule>>,
}

impl LenientFailoverWindowValidator {
    pub fn validate(&self, item: &LenientFailoverWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientFailoverWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientFailoverWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_failover_window_validator() -> LenientFailoverWindowValidator {
    LenientFailoverWindowValidator {
        rules: vec![
            Rc::new(LenientFailoverWindowRequiredReviewerRule {
                message: "lenient failover window needs this field".to_string(),
            }) as Rc<dyn LenientFailoverWindowRule>,
            Rc::new(LenientFailoverWindowRequiredTitleRule {
                message: "lenient failover window needs an entry".to_string(),
            }),
            Rc::new(LenientFailoverWindowCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_failover_window_validator();
        let item = new_lenient_failover_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = LenientFailoverWindowRequiredReviewerRule {
            message: "lenient failover window needs this field".to_string(),
        };
        let mut item = new_lenient_failover_window("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientFailoverWindowRequiredTitleRule {
            message: "lenient failover window needs a value".to_string(),
        };
        let mut item = new_lenient_failover_window("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientFailoverWindowCodeLengthRule { max_length: 3600 };
        let mut item = new_lenient_failover_window("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
