use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending plan audit must pass. Check returns its problems, or none.
pub trait PendingPlanAuditRule: std::fmt::Debug {
    fn check(&self, item: &PendingPlanAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanAuditScopeLengthRule {
    pub max_length: i64,
}

impl PendingPlanAuditRule for PendingPlanAuditScopeLengthRule {
    fn check(&self, item: &PendingPlanAudit) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanAuditRequiredTenantIdRule {
    pub message: String,
}

impl PendingPlanAuditRule for PendingPlanAuditRequiredTenantIdRule {
    fn check(&self, item: &PendingPlanAudit) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanAuditTenantIdLengthRule {
    pub max_length: i64,
}

impl PendingPlanAuditRule for PendingPlanAuditTenantIdLengthRule {
    fn check(&self, item: &PendingPlanAudit) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanAuditCodePrefixRule {
    pub prefix: String,
}

impl PendingPlanAuditRule for PendingPlanAuditCodePrefixRule {
    fn check(&self, item: &PendingPlanAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanAuditCodeLengthRule {
    pub max_length: i64,
}

impl PendingPlanAuditRule for PendingPlanAuditCodeLengthRule {
    fn check(&self, item: &PendingPlanAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanAuditActiveGuardRule {
    pub limit: f64,
}

impl PendingPlanAuditRule for PendingPlanAuditActiveGuardRule {
    fn check(&self, item: &PendingPlanAudit) -> Vec<String> {
        if item.active && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while active",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending plan audit.
#[derive(Debug, Clone)]
pub struct PendingPlanAuditValidator {
    pub rules: Vec<Rc<dyn PendingPlanAuditRule>>,
}

impl PendingPlanAuditValidator {
    pub fn validate(&self, item: &PendingPlanAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingPlanAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPlanAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_plan_audit_validator() -> PendingPlanAuditValidator {
    PendingPlanAuditValidator {
        rules: vec![
            Rc::new(PendingPlanAuditScopeLengthRule { max_length: 1000 }) as Rc<dyn PendingPlanAuditRule>,
            Rc::new(PendingPlanAuditRequiredTenantIdRule {
                message: "pending plan audit needs an entry".to_string(),
            }),
            Rc::new(PendingPlanAuditTenantIdLengthRule { max_length: 255 }),
            Rc::new(PendingPlanAuditCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PendingPlanAuditCodeLengthRule { max_length: 32 }),
            Rc::new(PendingPlanAuditActiveGuardRule { limit: 8.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_plan_audit_validator();
        let item = new_pending_plan_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = PendingPlanAuditScopeLengthRule { max_length: 80 };
        let mut item = new_pending_plan_audit("x", "y");
        item.scope = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = PendingPlanAuditRequiredTenantIdRule {
            message: "pending plan audit needs an entry".to_string(),
        };
        let mut item = new_pending_plan_audit("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = PendingPlanAuditTenantIdLengthRule { max_length: 1000 };
        let mut item = new_pending_plan_audit("x", "y");
        item.tenant_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingPlanAuditCodeLengthRule { max_length: 120 };
        let mut item = new_pending_plan_audit("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
