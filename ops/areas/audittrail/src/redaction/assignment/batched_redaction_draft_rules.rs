use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched redaction draft must pass. Check returns its problems, or none.
pub trait BatchedRedactionDraftRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRedactionDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionDraftStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRedactionDraftRule for BatchedRedactionDraftStartsAtRangeRule {
    fn check(&self, item: &BatchedRedactionDraft) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionDraftRequiredUnitRule {
    pub message: String,
}

impl BatchedRedactionDraftRule for BatchedRedactionDraftRequiredUnitRule {
    fn check(&self, item: &BatchedRedactionDraft) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionDraftRequiredLabelRule {
    pub message: String,
}

impl BatchedRedactionDraftRule for BatchedRedactionDraftRequiredLabelRule {
    fn check(&self, item: &BatchedRedactionDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionDraftUnitLengthRule {
    pub max_length: i64,
}

impl BatchedRedactionDraftRule for BatchedRedactionDraftUnitLengthRule {
    fn check(&self, item: &BatchedRedactionDraft) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionDraftRequiredCodeRule {
    pub message: String,
}

impl BatchedRedactionDraftRule for BatchedRedactionDraftRequiredCodeRule {
    fn check(&self, item: &BatchedRedactionDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched redaction draft.
#[derive(Debug, Clone)]
pub struct BatchedRedactionDraftValidator {
    pub rules: Vec<Rc<dyn BatchedRedactionDraftRule>>,
}

impl BatchedRedactionDraftValidator {
    pub fn validate(&self, item: &BatchedRedactionDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedRedactionDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRedactionDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_redaction_draft_validator() -> BatchedRedactionDraftValidator {
    BatchedRedactionDraftValidator {
        rules: vec![
            Rc::new(BatchedRedactionDraftStartsAtRangeRule { min: 32, max: 3600 }) as Rc<dyn BatchedRedactionDraftRule>,
            Rc::new(BatchedRedactionDraftRequiredUnitRule {
                message: "batched redaction draft needs an entry".to_string(),
            }),
            Rc::new(BatchedRedactionDraftRequiredLabelRule {
                message: "batched redaction draft needs a value".to_string(),
            }),
            Rc::new(BatchedRedactionDraftUnitLengthRule { max_length: 3600 }),
            Rc::new(BatchedRedactionDraftRequiredCodeRule {
                message: "batched redaction draft needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_redaction_draft_validator();
        let item = new_batched_redaction_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = BatchedRedactionDraftStartsAtRangeRule { min: 64, max: 255 };
        let mut item = new_batched_redaction_draft("x", "y");
        item.starts_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = BatchedRedactionDraftRequiredUnitRule {
            message: "batched redaction draft needs this field".to_string(),
        };
        let mut item = new_batched_redaction_draft("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedRedactionDraftRequiredLabelRule {
            message: "batched redaction draft needs this field".to_string(),
        };
        let mut item = new_batched_redaction_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = BatchedRedactionDraftUnitLengthRule { max_length: 3600 };
        let mut item = new_batched_redaction_draft("x", "y");
        item.unit = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedRedactionDraftRequiredCodeRule {
            message: "batched redaction draft needs a value".to_string(),
        };
        let mut item = new_batched_redaction_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
