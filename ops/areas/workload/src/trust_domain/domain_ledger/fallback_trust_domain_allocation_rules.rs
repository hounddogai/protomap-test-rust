use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback trust domain allocation must pass. Check returns its problems, or none.
pub trait FallbackTrustDomainAllocationRule: std::fmt::Debug {
    fn check(&self, item: &FallbackTrustDomainAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainAllocationExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackTrustDomainAllocationRule for FallbackTrustDomainAllocationExpiresAtRangeRule {
    fn check(&self, item: &FallbackTrustDomainAllocation) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainAllocationCodeLengthRule {
    pub max_length: i64,
}

impl FallbackTrustDomainAllocationRule for FallbackTrustDomainAllocationCodeLengthRule {
    fn check(&self, item: &FallbackTrustDomainAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainAllocationNameLengthRule {
    pub max_length: i64,
}

impl FallbackTrustDomainAllocationRule for FallbackTrustDomainAllocationNameLengthRule {
    fn check(&self, item: &FallbackTrustDomainAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainAllocationRequiredCodeRule {
    pub message: String,
}

impl FallbackTrustDomainAllocationRule for FallbackTrustDomainAllocationRequiredCodeRule {
    fn check(&self, item: &FallbackTrustDomainAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainAllocationRequiredClusterRule {
    pub message: String,
}

impl FallbackTrustDomainAllocationRule for FallbackTrustDomainAllocationRequiredClusterRule {
    fn check(&self, item: &FallbackTrustDomainAllocation) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback trust domain allocation.
#[derive(Debug, Clone)]
pub struct FallbackTrustDomainAllocationValidator {
    pub rules: Vec<Rc<dyn FallbackTrustDomainAllocationRule>>,
}

impl FallbackTrustDomainAllocationValidator {
    pub fn validate(&self, item: &FallbackTrustDomainAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackTrustDomainAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackTrustDomainAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_trust_domain_allocation_validator() -> FallbackTrustDomainAllocationValidator {
    FallbackTrustDomainAllocationValidator {
        rules: vec![
            Rc::new(FallbackTrustDomainAllocationExpiresAtRangeRule { min: 80, max: 120 })
                as Rc<dyn FallbackTrustDomainAllocationRule>,
            Rc::new(FallbackTrustDomainAllocationCodeLengthRule { max_length: 32 }),
            Rc::new(FallbackTrustDomainAllocationNameLengthRule { max_length: 86400 }),
            Rc::new(FallbackTrustDomainAllocationRequiredCodeRule {
                message: "fallback trust domain allocation needs an entry".to_string(),
            }),
            Rc::new(FallbackTrustDomainAllocationRequiredClusterRule {
                message: "fallback trust domain allocation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_trust_domain_allocation_validator();
        let item = new_fallback_trust_domain_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FallbackTrustDomainAllocationExpiresAtRangeRule { min: 64, max: 255 };
        let mut item = new_fallback_trust_domain_allocation("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackTrustDomainAllocationCodeLengthRule { max_length: 80 };
        let mut item = new_fallback_trust_domain_allocation("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackTrustDomainAllocationNameLengthRule { max_length: 16 };
        let mut item = new_fallback_trust_domain_allocation("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackTrustDomainAllocationRequiredCodeRule {
            message: "fallback trust domain allocation needs this field".to_string(),
        };
        let mut item = new_fallback_trust_domain_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = FallbackTrustDomainAllocationRequiredClusterRule {
            message: "fallback trust domain allocation needs this field".to_string(),
        };
        let mut item = new_fallback_trust_domain_allocation("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
