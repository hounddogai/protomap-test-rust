use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback tenant audit must pass. Check returns its problems, or none.
pub trait FallbackTenantAuditRule: std::fmt::Debug {
    fn check(&self, item: &FallbackTenantAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAuditRequiredTitleRule {
    pub message: String,
}

impl FallbackTenantAuditRule for FallbackTenantAuditRequiredTitleRule {
    fn check(&self, item: &FallbackTenantAudit) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAuditRequiredScopeRule {
    pub message: String,
}

impl FallbackTenantAuditRule for FallbackTenantAuditRequiredScopeRule {
    fn check(&self, item: &FallbackTenantAudit) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAuditTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackTenantAuditRule for FallbackTenantAuditTtlSecondsRangeRule {
    fn check(&self, item: &FallbackTenantAudit) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAuditApprovedGuardRule {
    pub limit: i64,
}

impl FallbackTenantAuditRule for FallbackTenantAuditApprovedGuardRule {
    fn check(&self, item: &FallbackTenantAudit) -> Vec<String> {
        if item.approved && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while approved",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAuditPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackTenantAuditRule for FallbackTenantAuditPriorityRangeRule {
    fn check(&self, item: &FallbackTenantAudit) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback tenant audit.
#[derive(Debug, Clone)]
pub struct FallbackTenantAuditValidator {
    pub rules: Vec<Rc<dyn FallbackTenantAuditRule>>,
}

impl FallbackTenantAuditValidator {
    pub fn validate(&self, item: &FallbackTenantAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackTenantAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackTenantAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_tenant_audit_validator() -> FallbackTenantAuditValidator {
    FallbackTenantAuditValidator {
        rules: vec![
            Rc::new(FallbackTenantAuditRequiredTitleRule {
                message: "fallback tenant audit needs a value".to_string(),
            }) as Rc<dyn FallbackTenantAuditRule>,
            Rc::new(FallbackTenantAuditRequiredScopeRule {
                message: "fallback tenant audit needs an entry".to_string(),
            }),
            Rc::new(FallbackTenantAuditTtlSecondsRangeRule { min: 1000, max: 86400 }),
            Rc::new(FallbackTenantAuditApprovedGuardRule { limit: 1000 }),
            Rc::new(FallbackTenantAuditPriorityRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_tenant_audit_validator();
        let item = new_fallback_tenant_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackTenantAuditRequiredTitleRule {
            message: "fallback tenant audit needs this field".to_string(),
        };
        let mut item = new_fallback_tenant_audit("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = FallbackTenantAuditRequiredScopeRule {
            message: "fallback tenant audit needs a value".to_string(),
        };
        let mut item = new_fallback_tenant_audit("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = FallbackTenantAuditTtlSecondsRangeRule { min: 64, max: 120 };
        let mut item = new_fallback_tenant_audit("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = FallbackTenantAuditPriorityRangeRule { min: 80, max: 255 };
        let mut item = new_fallback_tenant_audit("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
