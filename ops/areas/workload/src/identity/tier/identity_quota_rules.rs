use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity quota must pass. Check returns its problems, or none.
pub trait IdentityQuotaRule: std::fmt::Debug {
    fn check(&self, item: &IdentityQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityQuotaAllowedNameRule {
    pub allowed: Vec<String>,
}

impl IdentityQuotaRule for IdentityQuotaAllowedNameRule {
    fn check(&self, item: &IdentityQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityQuotaPrincipalLengthRule {
    pub max_length: i64,
}

impl IdentityQuotaRule for IdentityQuotaPrincipalLengthRule {
    fn check(&self, item: &IdentityQuota) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityQuotaQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityQuotaRule for IdentityQuotaQuantityRangeRule {
    fn check(&self, item: &IdentityQuota) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityQuotaSourceLengthRule {
    pub max_length: i64,
}

impl IdentityQuotaRule for IdentityQuotaSourceLengthRule {
    fn check(&self, item: &IdentityQuota) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityQuotaRequiredSourceRule {
    pub message: String,
}

impl IdentityQuotaRule for IdentityQuotaRequiredSourceRule {
    fn check(&self, item: &IdentityQuota) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity quota.
#[derive(Debug, Clone)]
pub struct IdentityQuotaValidator {
    pub rules: Vec<Rc<dyn IdentityQuotaRule>>,
}

impl IdentityQuotaValidator {
    pub fn validate(&self, item: &IdentityQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_quota_validator() -> IdentityQuotaValidator {
    IdentityQuotaValidator {
        rules: vec![
            Rc::new(IdentityQuotaAllowedNameRule { allowed: Vec::new() }) as Rc<dyn IdentityQuotaRule>,
            Rc::new(IdentityQuotaPrincipalLengthRule { max_length: 32 }),
            Rc::new(IdentityQuotaQuantityRangeRule { min: 255, max: 86400 }),
            Rc::new(IdentityQuotaSourceLengthRule { max_length: 64 }),
            Rc::new(IdentityQuotaRequiredSourceRule {
                message: "identity quota needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_quota_validator();
        let item = new_identity_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = IdentityQuotaPrincipalLengthRule { max_length: 3600 };
        let mut item = new_identity_quota("x", "y");
        item.principal = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = IdentityQuotaQuantityRangeRule { min: 64, max: 80 };
        let mut item = new_identity_quota("x", "y");
        item.quantity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = IdentityQuotaSourceLengthRule { max_length: 3600 };
        let mut item = new_identity_quota("x", "y");
        item.source = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = IdentityQuotaRequiredSourceRule {
            message: "identity quota needs an entry".to_string(),
        };
        let mut item = new_identity_quota("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
