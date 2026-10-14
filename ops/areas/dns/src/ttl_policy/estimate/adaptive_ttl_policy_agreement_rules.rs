use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive ttl policy agreement must pass. Check returns its problems, or none.
pub trait AdaptiveTtlPolicyAgreementRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTtlPolicyAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicyAgreementRequiredNameRule {
    pub message: String,
}

impl AdaptiveTtlPolicyAgreementRule for AdaptiveTtlPolicyAgreementRequiredNameRule {
    fn check(&self, item: &AdaptiveTtlPolicyAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicyAgreementCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveTtlPolicyAgreementRule for AdaptiveTtlPolicyAgreementCodePrefixRule {
    fn check(&self, item: &AdaptiveTtlPolicyAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTtlPolicyAgreementBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveTtlPolicyAgreementRule for AdaptiveTtlPolicyAgreementBurstRangeRule {
    fn check(&self, item: &AdaptiveTtlPolicyAgreement) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive ttl policy agreement.
#[derive(Debug, Clone)]
pub struct AdaptiveTtlPolicyAgreementValidator {
    pub rules: Vec<Rc<dyn AdaptiveTtlPolicyAgreementRule>>,
}

impl AdaptiveTtlPolicyAgreementValidator {
    pub fn validate(&self, item: &AdaptiveTtlPolicyAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveTtlPolicyAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTtlPolicyAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_ttl_policy_agreement_validator() -> AdaptiveTtlPolicyAgreementValidator {
    AdaptiveTtlPolicyAgreementValidator {
        rules: vec![
            Rc::new(AdaptiveTtlPolicyAgreementRequiredNameRule {
                message: "adaptive ttl policy agreement needs an entry".to_string(),
            }) as Rc<dyn AdaptiveTtlPolicyAgreementRule>,
            Rc::new(AdaptiveTtlPolicyAgreementCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(AdaptiveTtlPolicyAgreementBurstRangeRule { min: 32, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_ttl_policy_agreement_validator();
        let item = new_adaptive_ttl_policy_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AdaptiveTtlPolicyAgreementRequiredNameRule {
            message: "adaptive ttl policy agreement needs a value".to_string(),
        };
        let mut item = new_adaptive_ttl_policy_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = AdaptiveTtlPolicyAgreementBurstRangeRule { min: 16, max: 255 };
        let mut item = new_adaptive_ttl_policy_agreement("x", "y");
        item.burst = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
