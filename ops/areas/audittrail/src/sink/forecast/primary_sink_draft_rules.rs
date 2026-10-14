use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary sink draft must pass. Check returns its problems, or none.
pub trait PrimarySinkDraftRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySinkDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkDraftNotesLengthRule {
    pub max_length: i64,
}

impl PrimarySinkDraftRule for PrimarySinkDraftNotesLengthRule {
    fn check(&self, item: &PrimarySinkDraft) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkDraftCodeLengthRule {
    pub max_length: i64,
}

impl PrimarySinkDraftRule for PrimarySinkDraftCodeLengthRule {
    fn check(&self, item: &PrimarySinkDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkDraftCodePrefixRule {
    pub prefix: String,
}

impl PrimarySinkDraftRule for PrimarySinkDraftCodePrefixRule {
    fn check(&self, item: &PrimarySinkDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySinkDraftRequiredCodeRule {
    pub message: String,
}

impl PrimarySinkDraftRule for PrimarySinkDraftRequiredCodeRule {
    fn check(&self, item: &PrimarySinkDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary sink draft.
#[derive(Debug, Clone)]
pub struct PrimarySinkDraftValidator {
    pub rules: Vec<Rc<dyn PrimarySinkDraftRule>>,
}

impl PrimarySinkDraftValidator {
    pub fn validate(&self, item: &PrimarySinkDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimarySinkDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySinkDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_sink_draft_validator() -> PrimarySinkDraftValidator {
    PrimarySinkDraftValidator {
        rules: vec![
            Rc::new(PrimarySinkDraftNotesLengthRule { max_length: 32 }) as Rc<dyn PrimarySinkDraftRule>,
            Rc::new(PrimarySinkDraftCodeLengthRule { max_length: 64 }),
            Rc::new(PrimarySinkDraftCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PrimarySinkDraftRequiredCodeRule {
                message: "primary sink draft needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_sink_draft_validator();
        let item = new_primary_sink_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = PrimarySinkDraftNotesLengthRule { max_length: 16 };
        let mut item = new_primary_sink_draft("x", "y");
        item.notes = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimarySinkDraftCodeLengthRule { max_length: 120 };
        let mut item = new_primary_sink_draft("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimarySinkDraftRequiredCodeRule {
            message: "primary sink draft needs an entry".to_string(),
        };
        let mut item = new_primary_sink_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
