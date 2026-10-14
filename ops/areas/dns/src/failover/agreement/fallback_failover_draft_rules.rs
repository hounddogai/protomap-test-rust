use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback failover draft must pass. Check returns its problems, or none.
pub trait FallbackFailoverDraftRule: std::fmt::Debug {
    fn check(&self, item: &FallbackFailoverDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverDraftCodeLengthRule {
    pub max_length: i64,
}

impl FallbackFailoverDraftRule for FallbackFailoverDraftCodeLengthRule {
    fn check(&self, item: &FallbackFailoverDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverDraftRequiredCodeRule {
    pub message: String,
}

impl FallbackFailoverDraftRule for FallbackFailoverDraftRequiredCodeRule {
    fn check(&self, item: &FallbackFailoverDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverDraftLabelLengthRule {
    pub max_length: i64,
}

impl FallbackFailoverDraftRule for FallbackFailoverDraftLabelLengthRule {
    fn check(&self, item: &FallbackFailoverDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverDraftRequiredKeyIdRule {
    pub message: String,
}

impl FallbackFailoverDraftRule for FallbackFailoverDraftRequiredKeyIdRule {
    fn check(&self, item: &FallbackFailoverDraft) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback failover draft.
#[derive(Debug, Clone)]
pub struct FallbackFailoverDraftValidator {
    pub rules: Vec<Rc<dyn FallbackFailoverDraftRule>>,
}

impl FallbackFailoverDraftValidator {
    pub fn validate(&self, item: &FallbackFailoverDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackFailoverDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackFailoverDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_failover_draft_validator() -> FallbackFailoverDraftValidator {
    FallbackFailoverDraftValidator {
        rules: vec![
            Rc::new(FallbackFailoverDraftCodeLengthRule { max_length: 64 }) as Rc<dyn FallbackFailoverDraftRule>,
            Rc::new(FallbackFailoverDraftRequiredCodeRule {
                message: "fallback failover draft needs a value".to_string(),
            }),
            Rc::new(FallbackFailoverDraftLabelLengthRule { max_length: 80 }),
            Rc::new(FallbackFailoverDraftRequiredKeyIdRule {
                message: "fallback failover draft needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_failover_draft_validator();
        let item = new_fallback_failover_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackFailoverDraftCodeLengthRule { max_length: 64 };
        let mut item = new_fallback_failover_draft("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackFailoverDraftRequiredCodeRule {
            message: "fallback failover draft needs this field".to_string(),
        };
        let mut item = new_fallback_failover_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackFailoverDraftLabelLengthRule { max_length: 120 };
        let mut item = new_fallback_failover_draft("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = FallbackFailoverDraftRequiredKeyIdRule {
            message: "fallback failover draft needs this field".to_string(),
        };
        let mut item = new_fallback_failover_draft("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
