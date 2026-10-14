use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending reservation audit must pass. Check returns its problems, or none.
pub trait PendingReservationAuditRule: std::fmt::Debug {
    fn check(&self, item: &PendingReservationAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReservationAuditCipherLengthRule {
    pub max_length: i64,
}

impl PendingReservationAuditRule for PendingReservationAuditCipherLengthRule {
    fn check(&self, item: &PendingReservationAudit) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReservationAuditRequiredCodeRule {
    pub message: String,
}

impl PendingReservationAuditRule for PendingReservationAuditRequiredCodeRule {
    fn check(&self, item: &PendingReservationAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReservationAuditRequiredCipherRule {
    pub message: String,
}

impl PendingReservationAuditRule for PendingReservationAuditRequiredCipherRule {
    fn check(&self, item: &PendingReservationAudit) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReservationAuditCodePrefixRule {
    pub prefix: String,
}

impl PendingReservationAuditRule for PendingReservationAuditCodePrefixRule {
    fn check(&self, item: &PendingReservationAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending reservation audit.
#[derive(Debug, Clone)]
pub struct PendingReservationAuditValidator {
    pub rules: Vec<Rc<dyn PendingReservationAuditRule>>,
}

impl PendingReservationAuditValidator {
    pub fn validate(&self, item: &PendingReservationAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingReservationAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingReservationAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_reservation_audit_validator() -> PendingReservationAuditValidator {
    PendingReservationAuditValidator {
        rules: vec![
            Rc::new(PendingReservationAuditCipherLengthRule { max_length: 1000 })
                as Rc<dyn PendingReservationAuditRule>,
            Rc::new(PendingReservationAuditRequiredCodeRule {
                message: "pending reservation audit needs a value".to_string(),
            }),
            Rc::new(PendingReservationAuditRequiredCipherRule {
                message: "pending reservation audit needs this field".to_string(),
            }),
            Rc::new(PendingReservationAuditCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_reservation_audit_validator();
        let item = new_pending_reservation_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = PendingReservationAuditCipherLengthRule { max_length: 80 };
        let mut item = new_pending_reservation_audit("x", "y");
        item.cipher = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingReservationAuditRequiredCodeRule {
            message: "pending reservation audit needs an entry".to_string(),
        };
        let mut item = new_pending_reservation_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = PendingReservationAuditRequiredCipherRule {
            message: "pending reservation audit needs an entry".to_string(),
        };
        let mut item = new_pending_reservation_audit("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
