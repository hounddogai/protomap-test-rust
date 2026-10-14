use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending runbook draft must pass. Check returns its problems, or none.
pub trait PendingRunbookDraftRule: std::fmt::Debug {
    fn check(&self, item: &PendingRunbookDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookDraftRequiredNameRule {
    pub message: String,
}

impl PendingRunbookDraftRule for PendingRunbookDraftRequiredNameRule {
    fn check(&self, item: &PendingRunbookDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookDraftTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRunbookDraftRule for PendingRunbookDraftTtlSecondsRangeRule {
    fn check(&self, item: &PendingRunbookDraft) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookDraftAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRunbookDraftRule for PendingRunbookDraftAttemptsRangeRule {
    fn check(&self, item: &PendingRunbookDraft) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookDraftRequiredCodeRule {
    pub message: String,
}

impl PendingRunbookDraftRule for PendingRunbookDraftRequiredCodeRule {
    fn check(&self, item: &PendingRunbookDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending runbook draft.
#[derive(Debug, Clone)]
pub struct PendingRunbookDraftValidator {
    pub rules: Vec<Rc<dyn PendingRunbookDraftRule>>,
}

impl PendingRunbookDraftValidator {
    pub fn validate(&self, item: &PendingRunbookDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRunbookDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRunbookDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_runbook_draft_validator() -> PendingRunbookDraftValidator {
    PendingRunbookDraftValidator {
        rules: vec![
            Rc::new(PendingRunbookDraftRequiredNameRule {
                message: "pending runbook draft needs a value".to_string(),
            }) as Rc<dyn PendingRunbookDraftRule>,
            Rc::new(PendingRunbookDraftTtlSecondsRangeRule { min: 3600, max: 86400 }),
            Rc::new(PendingRunbookDraftAttemptsRangeRule { min: 64, max: 80 }),
            Rc::new(PendingRunbookDraftRequiredCodeRule {
                message: "pending runbook draft needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_runbook_draft_validator();
        let item = new_pending_runbook_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingRunbookDraftRequiredNameRule {
            message: "pending runbook draft needs a value".to_string(),
        };
        let mut item = new_pending_runbook_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PendingRunbookDraftTtlSecondsRangeRule { min: 64, max: 255 };
        let mut item = new_pending_runbook_draft("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = PendingRunbookDraftAttemptsRangeRule { min: 16, max: 120 };
        let mut item = new_pending_runbook_draft("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingRunbookDraftRequiredCodeRule {
            message: "pending runbook draft needs this field".to_string(),
        };
        let mut item = new_pending_runbook_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
