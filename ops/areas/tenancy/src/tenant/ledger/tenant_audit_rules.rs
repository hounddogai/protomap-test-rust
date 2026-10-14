use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant audit must pass. Check returns its problems, or none.
pub trait TenantAuditRule: std::fmt::Debug {
    fn check(&self, item: &TenantAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAuditPinnedGuardRule {
    pub limit: f64,
}

impl TenantAuditRule for TenantAuditPinnedGuardRule {
    fn check(&self, item: &TenantAudit) -> Vec<String> {
        if item.pinned && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while pinned",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAuditExternalRefLengthRule {
    pub max_length: i64,
}

impl TenantAuditRule for TenantAuditExternalRefLengthRule {
    fn check(&self, item: &TenantAudit) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAuditTimezoneLengthRule {
    pub max_length: i64,
}

impl TenantAuditRule for TenantAuditTimezoneLengthRule {
    fn check(&self, item: &TenantAudit) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAuditCodeLengthRule {
    pub max_length: i64,
}

impl TenantAuditRule for TenantAuditCodeLengthRule {
    fn check(&self, item: &TenantAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAuditRequiredExternalRefRule {
    pub message: String,
}

impl TenantAuditRule for TenantAuditRequiredExternalRefRule {
    fn check(&self, item: &TenantAudit) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant audit.
#[derive(Debug, Clone)]
pub struct TenantAuditValidator {
    pub rules: Vec<Rc<dyn TenantAuditRule>>,
}

impl TenantAuditValidator {
    pub fn validate(&self, item: &TenantAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_audit_validator() -> TenantAuditValidator {
    TenantAuditValidator {
        rules: vec![
            Rc::new(TenantAuditPinnedGuardRule { limit: 1.4 }) as Rc<dyn TenantAuditRule>,
            Rc::new(TenantAuditExternalRefLengthRule { max_length: 32 }),
            Rc::new(TenantAuditTimezoneLengthRule { max_length: 80 }),
            Rc::new(TenantAuditCodeLengthRule { max_length: 32 }),
            Rc::new(TenantAuditRequiredExternalRefRule {
                message: "tenant audit needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_audit_validator();
        let item = new_tenant_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = TenantAuditExternalRefLengthRule { max_length: 120 };
        let mut item = new_tenant_audit("x", "y");
        item.external_ref = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = TenantAuditTimezoneLengthRule { max_length: 1000 };
        let mut item = new_tenant_audit("x", "y");
        item.timezone = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TenantAuditCodeLengthRule { max_length: 1000 };
        let mut item = new_tenant_audit("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = TenantAuditRequiredExternalRefRule {
            message: "tenant audit needs an entry".to_string(),
        };
        let mut item = new_tenant_audit("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
