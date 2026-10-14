use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy reservation draft must pass. Check returns its problems, or none.
pub trait LegacyReservationDraftRule: std::fmt::Debug {
    fn check(&self, item: &LegacyReservationDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationDraftCodeLengthRule {
    pub max_length: i64,
}

impl LegacyReservationDraftRule for LegacyReservationDraftCodeLengthRule {
    fn check(&self, item: &LegacyReservationDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationDraftRequiredCodeRule {
    pub message: String,
}

impl LegacyReservationDraftRule for LegacyReservationDraftRequiredCodeRule {
    fn check(&self, item: &LegacyReservationDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationDraftRequiredLabelRule {
    pub message: String,
}

impl LegacyReservationDraftRule for LegacyReservationDraftRequiredLabelRule {
    fn check(&self, item: &LegacyReservationDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationDraftAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyReservationDraftRule for LegacyReservationDraftAllowedCodeRule {
    fn check(&self, item: &LegacyReservationDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationDraftLabelLengthRule {
    pub max_length: i64,
}

impl LegacyReservationDraftRule for LegacyReservationDraftLabelLengthRule {
    fn check(&self, item: &LegacyReservationDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyReservationDraftRequiredTenantIdRule {
    pub message: String,
}

impl LegacyReservationDraftRule for LegacyReservationDraftRequiredTenantIdRule {
    fn check(&self, item: &LegacyReservationDraft) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy reservation draft.
#[derive(Debug, Clone)]
pub struct LegacyReservationDraftValidator {
    pub rules: Vec<Rc<dyn LegacyReservationDraftRule>>,
}

impl LegacyReservationDraftValidator {
    pub fn validate(&self, item: &LegacyReservationDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyReservationDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyReservationDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_reservation_draft_validator() -> LegacyReservationDraftValidator {
    LegacyReservationDraftValidator {
        rules: vec![
            Rc::new(LegacyReservationDraftCodeLengthRule { max_length: 255 }) as Rc<dyn LegacyReservationDraftRule>,
            Rc::new(LegacyReservationDraftRequiredCodeRule {
                message: "legacy reservation draft needs an entry".to_string(),
            }),
            Rc::new(LegacyReservationDraftRequiredLabelRule {
                message: "legacy reservation draft needs a value".to_string(),
            }),
            Rc::new(LegacyReservationDraftAllowedCodeRule {
                allowed: vec!["global".to_string(), "pending".to_string()],
            }),
            Rc::new(LegacyReservationDraftLabelLengthRule { max_length: 1000 }),
            Rc::new(LegacyReservationDraftRequiredTenantIdRule {
                message: "legacy reservation draft needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_reservation_draft_validator();
        let item = new_legacy_reservation_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyReservationDraftCodeLengthRule { max_length: 120 };
        let mut item = new_legacy_reservation_draft("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyReservationDraftRequiredCodeRule {
            message: "legacy reservation draft needs this field".to_string(),
        };
        let mut item = new_legacy_reservation_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LegacyReservationDraftRequiredLabelRule {
            message: "legacy reservation draft needs an entry".to_string(),
        };
        let mut item = new_legacy_reservation_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyReservationDraftLabelLengthRule { max_length: 255 };
        let mut item = new_legacy_reservation_draft("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = LegacyReservationDraftRequiredTenantIdRule {
            message: "legacy reservation draft needs this field".to_string(),
        };
        let mut item = new_legacy_reservation_draft("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
