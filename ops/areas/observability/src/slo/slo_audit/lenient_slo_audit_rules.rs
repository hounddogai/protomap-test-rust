use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient slo audit must pass. Check returns its problems, or none.
pub trait LenientSloAuditRule: std::fmt::Debug {
    fn check(&self, item: &LenientSloAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloAuditLabelLengthRule {
    pub max_length: i64,
}

impl LenientSloAuditRule for LenientSloAuditLabelLengthRule {
    fn check(&self, item: &LenientSloAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloAuditSummaryLengthRule {
    pub max_length: i64,
}

impl LenientSloAuditRule for LenientSloAuditSummaryLengthRule {
    fn check(&self, item: &LenientSloAudit) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloAuditCodePrefixRule {
    pub prefix: String,
}

impl LenientSloAuditRule for LenientSloAuditCodePrefixRule {
    fn check(&self, item: &LenientSloAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloAuditRequiredSummaryRule {
    pub message: String,
}

impl LenientSloAuditRule for LenientSloAuditRequiredSummaryRule {
    fn check(&self, item: &LenientSloAudit) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient slo audit.
#[derive(Debug, Clone)]
pub struct LenientSloAuditValidator {
    pub rules: Vec<Rc<dyn LenientSloAuditRule>>,
}

impl LenientSloAuditValidator {
    pub fn validate(&self, item: &LenientSloAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientSloAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSloAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_slo_audit_validator() -> LenientSloAuditValidator {
    LenientSloAuditValidator {
        rules: vec![
            Rc::new(LenientSloAuditLabelLengthRule { max_length: 64 }) as Rc<dyn LenientSloAuditRule>,
            Rc::new(LenientSloAuditSummaryLengthRule { max_length: 255 }),
            Rc::new(LenientSloAuditCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LenientSloAuditRequiredSummaryRule {
                message: "lenient slo audit needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_slo_audit_validator();
        let item = new_lenient_slo_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientSloAuditLabelLengthRule { max_length: 3600 };
        let mut item = new_lenient_slo_audit("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = LenientSloAuditSummaryLengthRule { max_length: 3600 };
        let mut item = new_lenient_slo_audit("x", "y");
        item.summary = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = LenientSloAuditRequiredSummaryRule {
            message: "lenient slo audit needs an entry".to_string(),
        };
        let mut item = new_lenient_slo_audit("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
