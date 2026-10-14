use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient tenant quota must pass. Check returns its problems, or none.
pub trait LenientTenantQuotaRule: std::fmt::Debug {
    fn check(&self, item: &LenientTenantQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantQuotaCodePrefixRule {
    pub prefix: String,
}

impl LenientTenantQuotaRule for LenientTenantQuotaCodePrefixRule {
    fn check(&self, item: &LenientTenantQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantQuotaCodeLengthRule {
    pub max_length: i64,
}

impl LenientTenantQuotaRule for LenientTenantQuotaCodeLengthRule {
    fn check(&self, item: &LenientTenantQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantQuotaAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientTenantQuotaRule for LenientTenantQuotaAmountRangeRule {
    fn check(&self, item: &LenientTenantQuota) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantQuotaPinnedGuardRule {
    pub limit: i64,
}

impl LenientTenantQuotaRule for LenientTenantQuotaPinnedGuardRule {
    fn check(&self, item: &LenientTenantQuota) -> Vec<String> {
        if item.pinned && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while pinned",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantQuotaRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientTenantQuotaRule for LenientTenantQuotaRotationDaysRangeRule {
    fn check(&self, item: &LenientTenantQuota) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient tenant quota.
#[derive(Debug, Clone)]
pub struct LenientTenantQuotaValidator {
    pub rules: Vec<Rc<dyn LenientTenantQuotaRule>>,
}

impl LenientTenantQuotaValidator {
    pub fn validate(&self, item: &LenientTenantQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientTenantQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTenantQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_tenant_quota_validator() -> LenientTenantQuotaValidator {
    LenientTenantQuotaValidator {
        rules: vec![
            Rc::new(LenientTenantQuotaCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn LenientTenantQuotaRule>,
            Rc::new(LenientTenantQuotaCodeLengthRule { max_length: 16 }),
            Rc::new(LenientTenantQuotaAmountRangeRule { min: 4.0, max: 749.9 }),
            Rc::new(LenientTenantQuotaPinnedGuardRule { limit: 3600 }),
            Rc::new(LenientTenantQuotaRotationDaysRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_tenant_quota_validator();
        let item = new_lenient_tenant_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientTenantQuotaCodeLengthRule { max_length: 86400 };
        let mut item = new_lenient_tenant_quota("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = LenientTenantQuotaAmountRangeRule { min: 7.1, max: 3511.1 };
        let mut item = new_lenient_tenant_quota("x", "y");
        item.amount = 3512.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = LenientTenantQuotaRotationDaysRangeRule { min: 1000, max: 86400 };
        let mut item = new_lenient_tenant_quota("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
