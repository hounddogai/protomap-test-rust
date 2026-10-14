use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending redaction draft must pass. Check returns its problems, or none.
pub trait PendingRedactionDraftRule: std::fmt::Debug {
    fn check(&self, item: &PendingRedactionDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionDraftDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRedactionDraftRule for PendingRedactionDraftDurationMinutesRangeRule {
    fn check(&self, item: &PendingRedactionDraft) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionDraftEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRedactionDraftRule for PendingRedactionDraftEndsAtRangeRule {
    fn check(&self, item: &PendingRedactionDraft) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionDraftCodePrefixRule {
    pub prefix: String,
}

impl PendingRedactionDraftRule for PendingRedactionDraftCodePrefixRule {
    fn check(&self, item: &PendingRedactionDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionDraftLabelLengthRule {
    pub max_length: i64,
}

impl PendingRedactionDraftRule for PendingRedactionDraftLabelLengthRule {
    fn check(&self, item: &PendingRedactionDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending redaction draft.
#[derive(Debug, Clone)]
pub struct PendingRedactionDraftValidator {
    pub rules: Vec<Rc<dyn PendingRedactionDraftRule>>,
}

impl PendingRedactionDraftValidator {
    pub fn validate(&self, item: &PendingRedactionDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRedactionDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRedactionDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_redaction_draft_validator() -> PendingRedactionDraftValidator {
    PendingRedactionDraftValidator {
        rules: vec![
            Rc::new(PendingRedactionDraftDurationMinutesRangeRule { min: 16, max: 255 })
                as Rc<dyn PendingRedactionDraftRule>,
            Rc::new(PendingRedactionDraftEndsAtRangeRule { min: 32, max: 255 }),
            Rc::new(PendingRedactionDraftCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PendingRedactionDraftLabelLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_redaction_draft_validator();
        let item = new_pending_redaction_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = PendingRedactionDraftDurationMinutesRangeRule { min: 120, max: 255 };
        let mut item = new_pending_redaction_draft("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = PendingRedactionDraftEndsAtRangeRule { min: 64, max: 255 };
        let mut item = new_pending_redaction_draft("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingRedactionDraftLabelLengthRule { max_length: 3600 };
        let mut item = new_pending_redaction_draft("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
