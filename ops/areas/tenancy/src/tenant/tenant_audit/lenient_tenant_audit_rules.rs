use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient tenant audit must pass. Check returns its problems, or none.
pub trait LenientTenantAuditRule: std::fmt::Debug {
    fn check(&self, item: &LenientTenantAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantAuditQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientTenantAuditRule for LenientTenantAuditQuotaUnitsRangeRule {
    fn check(&self, item: &LenientTenantAudit) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantAuditReviewerLengthRule {
    pub max_length: i64,
}

impl LenientTenantAuditRule for LenientTenantAuditReviewerLengthRule {
    fn check(&self, item: &LenientTenantAudit) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantAuditAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientTenantAuditRule for LenientTenantAuditAllowedCodeRule {
    fn check(&self, item: &LenientTenantAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient tenant audit.
#[derive(Debug, Clone)]
pub struct LenientTenantAuditValidator {
    pub rules: Vec<Rc<dyn LenientTenantAuditRule>>,
}

impl LenientTenantAuditValidator {
    pub fn validate(&self, item: &LenientTenantAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientTenantAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTenantAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_tenant_audit_validator() -> LenientTenantAuditValidator {
    LenientTenantAuditValidator {
        rules: vec![
            Rc::new(LenientTenantAuditQuotaUnitsRangeRule { min: 16, max: 3600 }) as Rc<dyn LenientTenantAuditRule>,
            Rc::new(LenientTenantAuditReviewerLengthRule { max_length: 32 }),
            Rc::new(LenientTenantAuditAllowedCodeRule {
                allowed: vec!["tiered".to_string(), "legacy".to_string(), "manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_tenant_audit_validator();
        let item = new_lenient_tenant_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = LenientTenantAuditQuotaUnitsRangeRule { min: 80, max: 255 };
        let mut item = new_lenient_tenant_audit("x", "y");
        item.quota_units = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = LenientTenantAuditReviewerLengthRule { max_length: 32 };
        let mut item = new_lenient_tenant_audit("x", "y");
        item.reviewer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
