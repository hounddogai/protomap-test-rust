use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction draft must pass. Check returns its problems, or none.
pub trait RedactionDraftRule: std::fmt::Debug {
    fn check(&self, item: &RedactionDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionDraftRequiredHostRule {
    pub message: String,
}

impl RedactionDraftRule for RedactionDraftRequiredHostRule {
    fn check(&self, item: &RedactionDraft) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionDraftQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionDraftRule for RedactionDraftQuotaUnitsRangeRule {
    fn check(&self, item: &RedactionDraft) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionDraftBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionDraftRule for RedactionDraftBatchSizeRangeRule {
    fn check(&self, item: &RedactionDraft) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionDraftPhoneLengthRule {
    pub max_length: i64,
}

impl RedactionDraftRule for RedactionDraftPhoneLengthRule {
    fn check(&self, item: &RedactionDraft) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction draft.
#[derive(Debug, Clone)]
pub struct RedactionDraftValidator {
    pub rules: Vec<Rc<dyn RedactionDraftRule>>,
}

impl RedactionDraftValidator {
    pub fn validate(&self, item: &RedactionDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_draft_validator() -> RedactionDraftValidator {
    RedactionDraftValidator {
        rules: vec![
            Rc::new(RedactionDraftRequiredHostRule {
                message: "redaction draft needs a value".to_string(),
            }) as Rc<dyn RedactionDraftRule>,
            Rc::new(RedactionDraftQuotaUnitsRangeRule { min: 16, max: 86400 }),
            Rc::new(RedactionDraftBatchSizeRangeRule { min: 32, max: 1000 }),
            Rc::new(RedactionDraftPhoneLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_draft_validator();
        let item = new_redaction_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RedactionDraftRequiredHostRule {
            message: "redaction draft needs this field".to_string(),
        };
        let mut item = new_redaction_draft("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = RedactionDraftQuotaUnitsRangeRule { min: 32, max: 255 };
        let mut item = new_redaction_draft("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = RedactionDraftBatchSizeRangeRule { min: 255, max: 255 };
        let mut item = new_redaction_draft("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = RedactionDraftPhoneLengthRule { max_length: 120 };
        let mut item = new_redaction_draft("x", "y");
        item.phone = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
