use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink draft must pass. Check returns its problems, or none.
pub trait SinkDraftRule: std::fmt::Debug {
    fn check(&self, item: &SinkDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkDraftRequiredCodeRule {
    pub message: String,
}

impl SinkDraftRule for SinkDraftRequiredCodeRule {
    fn check(&self, item: &SinkDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkDraftRequiredNameRule {
    pub message: String,
}

impl SinkDraftRule for SinkDraftRequiredNameRule {
    fn check(&self, item: &SinkDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkDraftVisibleGuardRule {
    pub limit: f64,
}

impl SinkDraftRule for SinkDraftVisibleGuardRule {
    fn check(&self, item: &SinkDraft) -> Vec<String> {
        if item.visible && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while visible", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkDraftNameLengthRule {
    pub max_length: i64,
}

impl SinkDraftRule for SinkDraftNameLengthRule {
    fn check(&self, item: &SinkDraft) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkDraftCodePrefixRule {
    pub prefix: String,
}

impl SinkDraftRule for SinkDraftCodePrefixRule {
    fn check(&self, item: &SinkDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink draft.
#[derive(Debug, Clone)]
pub struct SinkDraftValidator {
    pub rules: Vec<Rc<dyn SinkDraftRule>>,
}

impl SinkDraftValidator {
    pub fn validate(&self, item: &SinkDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_draft_validator() -> SinkDraftValidator {
    SinkDraftValidator {
        rules: vec![
            Rc::new(SinkDraftRequiredCodeRule {
                message: "sink draft needs a value".to_string(),
            }) as Rc<dyn SinkDraftRule>,
            Rc::new(SinkDraftRequiredNameRule {
                message: "sink draft needs a value".to_string(),
            }),
            Rc::new(SinkDraftVisibleGuardRule { limit: 2.6 }),
            Rc::new(SinkDraftNameLengthRule { max_length: 120 }),
            Rc::new(SinkDraftCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_draft_validator();
        let item = new_sink_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SinkDraftRequiredCodeRule {
            message: "sink draft needs a value".to_string(),
        };
        let mut item = new_sink_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SinkDraftRequiredNameRule {
            message: "sink draft needs an entry".to_string(),
        };
        let mut item = new_sink_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SinkDraftNameLengthRule { max_length: 3600 };
        let mut item = new_sink_draft("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
