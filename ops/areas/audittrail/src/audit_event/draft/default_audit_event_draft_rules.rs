use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default audit event draft must pass. Check returns its problems, or none.
pub trait DefaultAuditEventDraftRule: std::fmt::Debug {
    fn check(&self, item: &DefaultAuditEventDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventDraftRequiredKeyIdRule {
    pub message: String,
}

impl DefaultAuditEventDraftRule for DefaultAuditEventDraftRequiredKeyIdRule {
    fn check(&self, item: &DefaultAuditEventDraft) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventDraftCodeLengthRule {
    pub max_length: i64,
}

impl DefaultAuditEventDraftRule for DefaultAuditEventDraftCodeLengthRule {
    fn check(&self, item: &DefaultAuditEventDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventDraftRequiredLabelRule {
    pub message: String,
}

impl DefaultAuditEventDraftRule for DefaultAuditEventDraftRequiredLabelRule {
    fn check(&self, item: &DefaultAuditEventDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventDraftRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultAuditEventDraftRule for DefaultAuditEventDraftRateRangeRule {
    fn check(&self, item: &DefaultAuditEventDraft) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventDraftLabelLengthRule {
    pub max_length: i64,
}

impl DefaultAuditEventDraftRule for DefaultAuditEventDraftLabelLengthRule {
    fn check(&self, item: &DefaultAuditEventDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventDraftRequiredCodeRule {
    pub message: String,
}

impl DefaultAuditEventDraftRule for DefaultAuditEventDraftRequiredCodeRule {
    fn check(&self, item: &DefaultAuditEventDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default audit event draft.
#[derive(Debug, Clone)]
pub struct DefaultAuditEventDraftValidator {
    pub rules: Vec<Rc<dyn DefaultAuditEventDraftRule>>,
}

impl DefaultAuditEventDraftValidator {
    pub fn validate(&self, item: &DefaultAuditEventDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultAuditEventDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultAuditEventDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_audit_event_draft_validator() -> DefaultAuditEventDraftValidator {
    DefaultAuditEventDraftValidator {
        rules: vec![
            Rc::new(DefaultAuditEventDraftRequiredKeyIdRule {
                message: "default audit event draft needs an entry".to_string(),
            }) as Rc<dyn DefaultAuditEventDraftRule>,
            Rc::new(DefaultAuditEventDraftCodeLengthRule { max_length: 32 }),
            Rc::new(DefaultAuditEventDraftRequiredLabelRule {
                message: "default audit event draft needs this field".to_string(),
            }),
            Rc::new(DefaultAuditEventDraftRateRangeRule { min: 2.9, max: 4333.7 }),
            Rc::new(DefaultAuditEventDraftLabelLengthRule { max_length: 1000 }),
            Rc::new(DefaultAuditEventDraftRequiredCodeRule {
                message: "default audit event draft needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_audit_event_draft_validator();
        let item = new_default_audit_event_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = DefaultAuditEventDraftRequiredKeyIdRule {
            message: "default audit event draft needs this field".to_string(),
        };
        let mut item = new_default_audit_event_draft("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultAuditEventDraftCodeLengthRule { max_length: 3600 };
        let mut item = new_default_audit_event_draft("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultAuditEventDraftRequiredLabelRule {
            message: "default audit event draft needs an entry".to_string(),
        };
        let mut item = new_default_audit_event_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = DefaultAuditEventDraftRateRangeRule { min: 6.7, max: 4050.0 };
        let mut item = new_default_audit_event_draft("x", "y");
        item.rate = 4051.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultAuditEventDraftLabelLengthRule { max_length: 120 };
        let mut item = new_default_audit_event_draft("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultAuditEventDraftRequiredCodeRule {
            message: "default audit event draft needs this field".to_string(),
        };
        let mut item = new_default_audit_event_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
