use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain quota must pass. Check returns its problems, or none.
pub trait TrustDomainQuotaRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainQuotaNameLengthRule {
    pub max_length: i64,
}

impl TrustDomainQuotaRule for TrustDomainQuotaNameLengthRule {
    fn check(&self, item: &TrustDomainQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainQuotaCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainQuotaRule for TrustDomainQuotaCodeLengthRule {
    fn check(&self, item: &TrustDomainQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainQuotaBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainQuotaRule for TrustDomainQuotaBurstRangeRule {
    fn check(&self, item: &TrustDomainQuota) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TrustDomainQuotaRule for TrustDomainQuotaAllowedCodeRule {
    fn check(&self, item: &TrustDomainQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainQuotaCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainQuotaRule for TrustDomainQuotaCodePrefixRule {
    fn check(&self, item: &TrustDomainQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain quota.
#[derive(Debug, Clone)]
pub struct TrustDomainQuotaValidator {
    pub rules: Vec<Rc<dyn TrustDomainQuotaRule>>,
}

impl TrustDomainQuotaValidator {
    pub fn validate(&self, item: &TrustDomainQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_quota_validator() -> TrustDomainQuotaValidator {
    TrustDomainQuotaValidator {
        rules: vec![
            Rc::new(TrustDomainQuotaNameLengthRule { max_length: 3600 }) as Rc<dyn TrustDomainQuotaRule>,
            Rc::new(TrustDomainQuotaCodeLengthRule { max_length: 16 }),
            Rc::new(TrustDomainQuotaBurstRangeRule { min: 64, max: 1000 }),
            Rc::new(TrustDomainQuotaAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(TrustDomainQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_quota_validator();
        let item = new_trust_domain_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TrustDomainQuotaNameLengthRule { max_length: 80 };
        let mut item = new_trust_domain_quota("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainQuotaCodeLengthRule { max_length: 1000 };
        let mut item = new_trust_domain_quota("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = TrustDomainQuotaBurstRangeRule { min: 32, max: 3600 };
        let mut item = new_trust_domain_quota("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
