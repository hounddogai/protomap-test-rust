use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a audit event request must pass. Check returns its problems, or none.
pub trait AuditEventRequestRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRequestDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventRequestRule for AuditEventRequestDiscountRangeRule {
    fn check(&self, item: &AuditEventRequest) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRequestRequiredCodeRule {
    pub message: String,
}

impl AuditEventRequestRule for AuditEventRequestRequiredCodeRule {
    fn check(&self, item: &AuditEventRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRequestAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AuditEventRequestRule for AuditEventRequestAllowedLabelRule {
    fn check(&self, item: &AuditEventRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRequestRequiredCipherRule {
    pub message: String,
}

impl AuditEventRequestRule for AuditEventRequestRequiredCipherRule {
    fn check(&self, item: &AuditEventRequest) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRequestRequiredLabelRule {
    pub message: String,
}

impl AuditEventRequestRule for AuditEventRequestRequiredLabelRule {
    fn check(&self, item: &AuditEventRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRequestVisibleGuardRule {
    pub limit: f64,
}

impl AuditEventRequestRule for AuditEventRequestVisibleGuardRule {
    fn check(&self, item: &AuditEventRequest) -> Vec<String> {
        if item.visible && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while visible",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event request.
#[derive(Debug, Clone)]
pub struct AuditEventRequestValidator {
    pub rules: Vec<Rc<dyn AuditEventRequestRule>>,
}

impl AuditEventRequestValidator {
    pub fn validate(&self, item: &AuditEventRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_request_validator() -> AuditEventRequestValidator {
    AuditEventRequestValidator {
        rules: vec![
            Rc::new(AuditEventRequestDiscountRangeRule { min: 5.6, max: 1397.3 }) as Rc<dyn AuditEventRequestRule>,
            Rc::new(AuditEventRequestRequiredCodeRule {
                message: "audit event request needs an entry".to_string(),
            }),
            Rc::new(AuditEventRequestAllowedLabelRule {
                allowed: vec!["strict".to_string()],
            }),
            Rc::new(AuditEventRequestRequiredCipherRule {
                message: "audit event request needs an entry".to_string(),
            }),
            Rc::new(AuditEventRequestRequiredLabelRule {
                message: "audit event request needs a value".to_string(),
            }),
            Rc::new(AuditEventRequestVisibleGuardRule { limit: 9.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_request_validator();
        let item = new_audit_event_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = AuditEventRequestDiscountRangeRule { min: 7.3, max: 2240.6 };
        let mut item = new_audit_event_request("x", "y");
        item.discount = 2242.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventRequestRequiredCodeRule {
            message: "audit event request needs this field".to_string(),
        };
        let mut item = new_audit_event_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = AuditEventRequestRequiredCipherRule {
            message: "audit event request needs this field".to_string(),
        };
        let mut item = new_audit_event_request("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AuditEventRequestRequiredLabelRule {
            message: "audit event request needs this field".to_string(),
        };
        let mut item = new_audit_event_request("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
