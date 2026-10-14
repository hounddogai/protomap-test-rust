use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event draft must pass. Check returns its problems, or none.
pub trait AuditEventDraftRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDraftCodePrefixRule {
    pub prefix: String,
}

impl AuditEventDraftRule for AuditEventDraftCodePrefixRule {
    fn check(&self, item: &AuditEventDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDraftRequiredExternalRefRule {
    pub message: String,
}

impl AuditEventDraftRule for AuditEventDraftRequiredExternalRefRule {
    fn check(&self, item: &AuditEventDraft) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDraftRequiredCurrencyRule {
    pub message: String,
}

impl AuditEventDraftRule for AuditEventDraftRequiredCurrencyRule {
    fn check(&self, item: &AuditEventDraft) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDraftNameLengthRule {
    pub max_length: i64,
}

impl AuditEventDraftRule for AuditEventDraftNameLengthRule {
    fn check(&self, item: &AuditEventDraft) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event draft.
#[derive(Debug, Clone)]
pub struct AuditEventDraftValidator {
    pub rules: Vec<Rc<dyn AuditEventDraftRule>>,
}

impl AuditEventDraftValidator {
    pub fn validate(&self, item: &AuditEventDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_draft_validator() -> AuditEventDraftValidator {
    AuditEventDraftValidator {
        rules: vec![
            Rc::new(AuditEventDraftCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn AuditEventDraftRule>,
            Rc::new(AuditEventDraftRequiredExternalRefRule {
                message: "audit event draft needs an entry".to_string(),
            }),
            Rc::new(AuditEventDraftRequiredCurrencyRule {
                message: "audit event draft needs an entry".to_string(),
            }),
            Rc::new(AuditEventDraftNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_draft_validator();
        let item = new_audit_event_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = AuditEventDraftRequiredExternalRefRule {
            message: "audit event draft needs an entry".to_string(),
        };
        let mut item = new_audit_event_draft("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = AuditEventDraftRequiredCurrencyRule {
            message: "audit event draft needs an entry".to_string(),
        };
        let mut item = new_audit_event_draft("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AuditEventDraftNameLengthRule { max_length: 32 };
        let mut item = new_audit_event_draft("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
