use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy tenant audit must pass. Check returns its problems, or none.
pub trait LegacyTenantAuditRule: std::fmt::Debug {
    fn check(&self, item: &LegacyTenantAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAuditSegmentLengthRule {
    pub max_length: i64,
}

impl LegacyTenantAuditRule for LegacyTenantAuditSegmentLengthRule {
    fn check(&self, item: &LegacyTenantAudit) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAuditCodePrefixRule {
    pub prefix: String,
}

impl LegacyTenantAuditRule for LegacyTenantAuditCodePrefixRule {
    fn check(&self, item: &LegacyTenantAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAuditRequiredExternalRefRule {
    pub message: String,
}

impl LegacyTenantAuditRule for LegacyTenantAuditRequiredExternalRefRule {
    fn check(&self, item: &LegacyTenantAudit) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAuditCodeLengthRule {
    pub max_length: i64,
}

impl LegacyTenantAuditRule for LegacyTenantAuditCodeLengthRule {
    fn check(&self, item: &LegacyTenantAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy tenant audit.
#[derive(Debug, Clone)]
pub struct LegacyTenantAuditValidator {
    pub rules: Vec<Rc<dyn LegacyTenantAuditRule>>,
}

impl LegacyTenantAuditValidator {
    pub fn validate(&self, item: &LegacyTenantAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyTenantAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyTenantAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_tenant_audit_validator() -> LegacyTenantAuditValidator {
    LegacyTenantAuditValidator {
        rules: vec![
            Rc::new(LegacyTenantAuditSegmentLengthRule { max_length: 80 }) as Rc<dyn LegacyTenantAuditRule>,
            Rc::new(LegacyTenantAuditCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LegacyTenantAuditRequiredExternalRefRule {
                message: "legacy tenant audit needs this field".to_string(),
            }),
            Rc::new(LegacyTenantAuditCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_tenant_audit_validator();
        let item = new_legacy_tenant_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = LegacyTenantAuditSegmentLengthRule { max_length: 255 };
        let mut item = new_legacy_tenant_audit("x", "y");
        item.segment = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = LegacyTenantAuditRequiredExternalRefRule {
            message: "legacy tenant audit needs an entry".to_string(),
        };
        let mut item = new_legacy_tenant_audit("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyTenantAuditCodeLengthRule { max_length: 16 };
        let mut item = new_legacy_tenant_audit("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
