use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched timeline policy must pass. Check returns its problems, or none.
pub trait BatchedTimelinePolicyRule: std::fmt::Debug {
    fn check(&self, item: &BatchedTimelinePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelinePolicyCodePrefixRule {
    pub prefix: String,
}

impl BatchedTimelinePolicyRule for BatchedTimelinePolicyCodePrefixRule {
    fn check(&self, item: &BatchedTimelinePolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelinePolicyRequiredPrincipalRule {
    pub message: String,
}

impl BatchedTimelinePolicyRule for BatchedTimelinePolicyRequiredPrincipalRule {
    fn check(&self, item: &BatchedTimelinePolicy) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelinePolicyRequiredContactEmailRule {
    pub message: String,
}

impl BatchedTimelinePolicyRule for BatchedTimelinePolicyRequiredContactEmailRule {
    fn check(&self, item: &BatchedTimelinePolicy) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelinePolicyRequiredCodeRule {
    pub message: String,
}

impl BatchedTimelinePolicyRule for BatchedTimelinePolicyRequiredCodeRule {
    fn check(&self, item: &BatchedTimelinePolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelinePolicyTitleLengthRule {
    pub max_length: i64,
}

impl BatchedTimelinePolicyRule for BatchedTimelinePolicyTitleLengthRule {
    fn check(&self, item: &BatchedTimelinePolicy) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched timeline policy.
#[derive(Debug, Clone)]
pub struct BatchedTimelinePolicyValidator {
    pub rules: Vec<Rc<dyn BatchedTimelinePolicyRule>>,
}

impl BatchedTimelinePolicyValidator {
    pub fn validate(&self, item: &BatchedTimelinePolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedTimelinePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedTimelinePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_timeline_policy_validator() -> BatchedTimelinePolicyValidator {
    BatchedTimelinePolicyValidator {
        rules: vec![
            Rc::new(BatchedTimelinePolicyCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn BatchedTimelinePolicyRule>,
            Rc::new(BatchedTimelinePolicyRequiredPrincipalRule {
                message: "batched timeline policy needs an entry".to_string(),
            }),
            Rc::new(BatchedTimelinePolicyRequiredContactEmailRule {
                message: "batched timeline policy needs this field".to_string(),
            }),
            Rc::new(BatchedTimelinePolicyRequiredCodeRule {
                message: "batched timeline policy needs a value".to_string(),
            }),
            Rc::new(BatchedTimelinePolicyTitleLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_timeline_policy_validator();
        let item = new_batched_timeline_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = BatchedTimelinePolicyRequiredPrincipalRule {
            message: "batched timeline policy needs an entry".to_string(),
        };
        let mut item = new_batched_timeline_policy("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = BatchedTimelinePolicyRequiredContactEmailRule {
            message: "batched timeline policy needs a value".to_string(),
        };
        let mut item = new_batched_timeline_policy("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedTimelinePolicyRequiredCodeRule {
            message: "batched timeline policy needs a value".to_string(),
        };
        let mut item = new_batched_timeline_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BatchedTimelinePolicyTitleLengthRule { max_length: 32 };
        let mut item = new_batched_timeline_policy("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
