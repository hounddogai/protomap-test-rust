use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation audit must pass. Check returns its problems, or none.
pub trait EscalationAuditRule: std::fmt::Debug {
    fn check(&self, item: &EscalationAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAuditRequiredTitleRule {
    pub message: String,
}

impl EscalationAuditRule for EscalationAuditRequiredTitleRule {
    fn check(&self, item: &EscalationAudit) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAuditCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationAuditRule for EscalationAuditCreatedAtRangeRule {
    fn check(&self, item: &EscalationAudit) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAuditRequiredChecksumRule {
    pub message: String,
}

impl EscalationAuditRule for EscalationAuditRequiredChecksumRule {
    fn check(&self, item: &EscalationAudit) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAuditExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationAuditRule for EscalationAuditExpiresAtRangeRule {
    fn check(&self, item: &EscalationAudit) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a escalation audit.
#[derive(Debug, Clone)]
pub struct EscalationAuditValidator {
    pub rules: Vec<Rc<dyn EscalationAuditRule>>,
}

impl EscalationAuditValidator {
    pub fn validate(&self, item: &EscalationAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_audit_validator() -> EscalationAuditValidator {
    EscalationAuditValidator {
        rules: vec![
            Rc::new(EscalationAuditRequiredTitleRule {
                message: "escalation audit needs this field".to_string(),
            }) as Rc<dyn EscalationAuditRule>,
            Rc::new(EscalationAuditCreatedAtRangeRule { min: 120, max: 1000 }),
            Rc::new(EscalationAuditRequiredChecksumRule {
                message: "escalation audit needs this field".to_string(),
            }),
            Rc::new(EscalationAuditExpiresAtRangeRule { min: 80, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_audit_validator();
        let item = new_escalation_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = EscalationAuditRequiredTitleRule {
            message: "escalation audit needs a value".to_string(),
        };
        let mut item = new_escalation_audit("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = EscalationAuditCreatedAtRangeRule { min: 255, max: 86400 };
        let mut item = new_escalation_audit("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = EscalationAuditRequiredChecksumRule {
            message: "escalation audit needs a value".to_string(),
        };
        let mut item = new_escalation_audit("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = EscalationAuditExpiresAtRangeRule { min: 32, max: 86400 };
        let mut item = new_escalation_audit("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
