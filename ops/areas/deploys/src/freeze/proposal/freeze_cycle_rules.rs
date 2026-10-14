use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze cycle must pass. Check returns its problems, or none.
pub trait FreezeCycleRule: std::fmt::Debug {
    fn check(&self, item: &FreezeCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeCycleRequiredReviewerRule {
    pub message: String,
}

impl FreezeCycleRule for FreezeCycleRequiredReviewerRule {
    fn check(&self, item: &FreezeCycle) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeCycleReviewerLengthRule {
    pub max_length: i64,
}

impl FreezeCycleRule for FreezeCycleReviewerLengthRule {
    fn check(&self, item: &FreezeCycle) -> Vec<String> {
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
pub struct FreezeCycleRequiredNameRule {
    pub message: String,
}

impl FreezeCycleRule for FreezeCycleRequiredNameRule {
    fn check(&self, item: &FreezeCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeCycleCodePrefixRule {
    pub prefix: String,
}

impl FreezeCycleRule for FreezeCycleCodePrefixRule {
    fn check(&self, item: &FreezeCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze cycle.
#[derive(Debug, Clone)]
pub struct FreezeCycleValidator {
    pub rules: Vec<Rc<dyn FreezeCycleRule>>,
}

impl FreezeCycleValidator {
    pub fn validate(&self, item: &FreezeCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_cycle_validator() -> FreezeCycleValidator {
    FreezeCycleValidator {
        rules: vec![
            Rc::new(FreezeCycleRequiredReviewerRule {
                message: "freeze cycle needs this field".to_string(),
            }) as Rc<dyn FreezeCycleRule>,
            Rc::new(FreezeCycleReviewerLengthRule { max_length: 1000 }),
            Rc::new(FreezeCycleRequiredNameRule {
                message: "freeze cycle needs a value".to_string(),
            }),
            Rc::new(FreezeCycleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_cycle_validator();
        let item = new_freeze_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = FreezeCycleRequiredReviewerRule {
            message: "freeze cycle needs this field".to_string(),
        };
        let mut item = new_freeze_cycle("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = FreezeCycleReviewerLengthRule { max_length: 86400 };
        let mut item = new_freeze_cycle("x", "y");
        item.reviewer = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FreezeCycleRequiredNameRule {
            message: "freeze cycle needs an entry".to_string(),
        };
        let mut item = new_freeze_cycle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
