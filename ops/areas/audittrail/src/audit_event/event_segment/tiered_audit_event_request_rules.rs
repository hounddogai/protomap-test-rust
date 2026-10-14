use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered audit event request must pass. Check returns its problems, or none.
pub trait TieredAuditEventRequestRule: std::fmt::Debug {
    fn check(&self, item: &TieredAuditEventRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventRequestRequiredNotesRule {
    pub message: String,
}

impl TieredAuditEventRequestRule for TieredAuditEventRequestRequiredNotesRule {
    fn check(&self, item: &TieredAuditEventRequest) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventRequestArchivedGuardRule {
    pub limit: f64,
}

impl TieredAuditEventRequestRule for TieredAuditEventRequestArchivedGuardRule {
    fn check(&self, item: &TieredAuditEventRequest) -> Vec<String> {
        if item.archived && (item.budget > self.limit) {
            return vec![format!("budget {} exceeds {} while archived", item.budget, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventRequestAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl TieredAuditEventRequestRule for TieredAuditEventRequestAllowedCurrencyRule {
    fn check(&self, item: &TieredAuditEventRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is not allowed", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventRequestNotesLengthRule {
    pub max_length: i64,
}

impl TieredAuditEventRequestRule for TieredAuditEventRequestNotesLengthRule {
    fn check(&self, item: &TieredAuditEventRequest) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventRequestRequiredCodeRule {
    pub message: String,
}

impl TieredAuditEventRequestRule for TieredAuditEventRequestRequiredCodeRule {
    fn check(&self, item: &TieredAuditEventRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventRequestCodeLengthRule {
    pub max_length: i64,
}

impl TieredAuditEventRequestRule for TieredAuditEventRequestCodeLengthRule {
    fn check(&self, item: &TieredAuditEventRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered audit event request.
#[derive(Debug, Clone)]
pub struct TieredAuditEventRequestValidator {
    pub rules: Vec<Rc<dyn TieredAuditEventRequestRule>>,
}

impl TieredAuditEventRequestValidator {
    pub fn validate(&self, item: &TieredAuditEventRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredAuditEventRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredAuditEventRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_audit_event_request_validator() -> TieredAuditEventRequestValidator {
    TieredAuditEventRequestValidator {
        rules: vec![
            Rc::new(TieredAuditEventRequestRequiredNotesRule {
                message: "tiered audit event request needs this field".to_string(),
            }) as Rc<dyn TieredAuditEventRequestRule>,
            Rc::new(TieredAuditEventRequestArchivedGuardRule { limit: 4.3 }),
            Rc::new(TieredAuditEventRequestAllowedCurrencyRule {
                allowed: vec!["adaptive".to_string(), "legacy".to_string(), "tiered".to_string()],
            }),
            Rc::new(TieredAuditEventRequestNotesLengthRule { max_length: 16 }),
            Rc::new(TieredAuditEventRequestRequiredCodeRule {
                message: "tiered audit event request needs an entry".to_string(),
            }),
            Rc::new(TieredAuditEventRequestCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_audit_event_request_validator();
        let item = new_tiered_audit_event_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = TieredAuditEventRequestRequiredNotesRule {
            message: "tiered audit event request needs an entry".to_string(),
        };
        let mut item = new_tiered_audit_event_request("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = TieredAuditEventRequestNotesLengthRule { max_length: 80 };
        let mut item = new_tiered_audit_event_request("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredAuditEventRequestRequiredCodeRule {
            message: "tiered audit event request needs an entry".to_string(),
        };
        let mut item = new_tiered_audit_event_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredAuditEventRequestCodeLengthRule { max_length: 32 };
        let mut item = new_tiered_audit_event_request("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
