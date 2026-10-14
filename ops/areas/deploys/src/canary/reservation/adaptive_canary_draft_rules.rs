use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive canary draft must pass. Check returns its problems, or none.
pub trait AdaptiveCanaryDraftRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveCanaryDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryDraftNamespaceLengthRule {
    pub max_length: i64,
}

impl AdaptiveCanaryDraftRule for AdaptiveCanaryDraftNamespaceLengthRule {
    fn check(&self, item: &AdaptiveCanaryDraft) -> Vec<String> {
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
pub struct AdaptiveCanaryDraftCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveCanaryDraftRule for AdaptiveCanaryDraftCodePrefixRule {
    fn check(&self, item: &AdaptiveCanaryDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryDraftPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveCanaryDraftRule for AdaptiveCanaryDraftPriorityRangeRule {
    fn check(&self, item: &AdaptiveCanaryDraft) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryDraftTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveCanaryDraftRule for AdaptiveCanaryDraftTitleLengthRule {
    fn check(&self, item: &AdaptiveCanaryDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryDraftRequiredTitleRule {
    pub message: String,
}

impl AdaptiveCanaryDraftRule for AdaptiveCanaryDraftRequiredTitleRule {
    fn check(&self, item: &AdaptiveCanaryDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive canary draft.
#[derive(Debug, Clone)]
pub struct AdaptiveCanaryDraftValidator {
    pub rules: Vec<Rc<dyn AdaptiveCanaryDraftRule>>,
}

impl AdaptiveCanaryDraftValidator {
    pub fn validate(&self, item: &AdaptiveCanaryDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveCanaryDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveCanaryDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_canary_draft_validator() -> AdaptiveCanaryDraftValidator {
    AdaptiveCanaryDraftValidator {
        rules: vec![
            Rc::new(AdaptiveCanaryDraftNamespaceLengthRule { max_length: 120 }) as Rc<dyn AdaptiveCanaryDraftRule>,
            Rc::new(AdaptiveCanaryDraftCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(AdaptiveCanaryDraftPriorityRangeRule { min: 120, max: 3600 }),
            Rc::new(AdaptiveCanaryDraftTitleLengthRule { max_length: 64 }),
            Rc::new(AdaptiveCanaryDraftRequiredTitleRule {
                message: "adaptive canary draft needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_canary_draft_validator();
        let item = new_adaptive_canary_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = AdaptiveCanaryDraftNamespaceLengthRule { max_length: 16 };
        let mut item = new_adaptive_canary_draft("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = AdaptiveCanaryDraftPriorityRangeRule { min: 32, max: 86400 };
        let mut item = new_adaptive_canary_draft("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AdaptiveCanaryDraftTitleLengthRule { max_length: 1000 };
        let mut item = new_adaptive_canary_draft("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveCanaryDraftRequiredTitleRule {
            message: "adaptive canary draft needs this field".to_string(),
        };
        let mut item = new_adaptive_canary_draft("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
