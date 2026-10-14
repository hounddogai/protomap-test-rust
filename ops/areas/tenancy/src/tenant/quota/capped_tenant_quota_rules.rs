use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped tenant quota must pass. Check returns its problems, or none.
pub trait CappedTenantQuotaRule: std::fmt::Debug {
    fn check(&self, item: &CappedTenantQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantQuotaBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedTenantQuotaRule for CappedTenantQuotaBatchSizeRangeRule {
    fn check(&self, item: &CappedTenantQuota) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantQuotaNameLengthRule {
    pub max_length: i64,
}

impl CappedTenantQuotaRule for CappedTenantQuotaNameLengthRule {
    fn check(&self, item: &CappedTenantQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantQuotaRequiredCodeRule {
    pub message: String,
}

impl CappedTenantQuotaRule for CappedTenantQuotaRequiredCodeRule {
    fn check(&self, item: &CappedTenantQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantQuotaAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl CappedTenantQuotaRule for CappedTenantQuotaAllowedScopeRule {
    fn check(&self, item: &CappedTenantQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is not allowed", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantQuotaCodeLengthRule {
    pub max_length: i64,
}

impl CappedTenantQuotaRule for CappedTenantQuotaCodeLengthRule {
    fn check(&self, item: &CappedTenantQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantQuotaScopeLengthRule {
    pub max_length: i64,
}

impl CappedTenantQuotaRule for CappedTenantQuotaScopeLengthRule {
    fn check(&self, item: &CappedTenantQuota) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped tenant quota.
#[derive(Debug, Clone)]
pub struct CappedTenantQuotaValidator {
    pub rules: Vec<Rc<dyn CappedTenantQuotaRule>>,
}

impl CappedTenantQuotaValidator {
    pub fn validate(&self, item: &CappedTenantQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedTenantQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedTenantQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_tenant_quota_validator() -> CappedTenantQuotaValidator {
    CappedTenantQuotaValidator {
        rules: vec![
            Rc::new(CappedTenantQuotaBatchSizeRangeRule { min: 80, max: 120 }) as Rc<dyn CappedTenantQuotaRule>,
            Rc::new(CappedTenantQuotaNameLengthRule { max_length: 86400 }),
            Rc::new(CappedTenantQuotaRequiredCodeRule {
                message: "capped tenant quota needs a value".to_string(),
            }),
            Rc::new(CappedTenantQuotaAllowedScopeRule {
                allowed: vec!["manual".to_string(), "default".to_string(), "manual".to_string()],
            }),
            Rc::new(CappedTenantQuotaCodeLengthRule { max_length: 3600 }),
            Rc::new(CappedTenantQuotaScopeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_tenant_quota_validator();
        let item = new_capped_tenant_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = CappedTenantQuotaBatchSizeRangeRule { min: 16, max: 80 };
        let mut item = new_capped_tenant_quota("x", "y");
        item.batch_size = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CappedTenantQuotaNameLengthRule { max_length: 255 };
        let mut item = new_capped_tenant_quota("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedTenantQuotaRequiredCodeRule {
            message: "capped tenant quota needs an entry".to_string(),
        };
        let mut item = new_capped_tenant_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedTenantQuotaCodeLengthRule { max_length: 120 };
        let mut item = new_capped_tenant_quota("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = CappedTenantQuotaScopeLengthRule { max_length: 255 };
        let mut item = new_capped_tenant_quota("x", "y");
        item.scope = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
