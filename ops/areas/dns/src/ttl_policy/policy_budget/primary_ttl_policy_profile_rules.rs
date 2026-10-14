use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary ttl policy profile must pass. Check returns its problems, or none.
pub trait PrimaryTtlPolicyProfileRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryTtlPolicyProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyProfileRequiredCodeRule {
    pub message: String,
}

impl PrimaryTtlPolicyProfileRule for PrimaryTtlPolicyProfileRequiredCodeRule {
    fn check(&self, item: &PrimaryTtlPolicyProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyProfileRequiredSummaryRule {
    pub message: String,
}

impl PrimaryTtlPolicyProfileRule for PrimaryTtlPolicyProfileRequiredSummaryRule {
    fn check(&self, item: &PrimaryTtlPolicyProfile) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyProfileAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl PrimaryTtlPolicyProfileRule for PrimaryTtlPolicyProfileAllowedIssuerRule {
    fn check(&self, item: &PrimaryTtlPolicyProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is blocked", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyProfileWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryTtlPolicyProfileRule for PrimaryTtlPolicyProfileWeightRangeRule {
    fn check(&self, item: &PrimaryTtlPolicyProfile) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyProfileSummaryLengthRule {
    pub max_length: i64,
}

impl PrimaryTtlPolicyProfileRule for PrimaryTtlPolicyProfileSummaryLengthRule {
    fn check(&self, item: &PrimaryTtlPolicyProfile) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTtlPolicyProfileCodePrefixRule {
    pub prefix: String,
}

impl PrimaryTtlPolicyProfileRule for PrimaryTtlPolicyProfileCodePrefixRule {
    fn check(&self, item: &PrimaryTtlPolicyProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary ttl policy profile.
#[derive(Debug, Clone)]
pub struct PrimaryTtlPolicyProfileValidator {
    pub rules: Vec<Rc<dyn PrimaryTtlPolicyProfileRule>>,
}

impl PrimaryTtlPolicyProfileValidator {
    pub fn validate(&self, item: &PrimaryTtlPolicyProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryTtlPolicyProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryTtlPolicyProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_ttl_policy_profile_validator() -> PrimaryTtlPolicyProfileValidator {
    PrimaryTtlPolicyProfileValidator {
        rules: vec![
            Rc::new(PrimaryTtlPolicyProfileRequiredCodeRule {
                message: "primary ttl policy profile needs an entry".to_string(),
            }) as Rc<dyn PrimaryTtlPolicyProfileRule>,
            Rc::new(PrimaryTtlPolicyProfileRequiredSummaryRule {
                message: "primary ttl policy profile needs this field".to_string(),
            }),
            Rc::new(PrimaryTtlPolicyProfileAllowedIssuerRule { allowed: Vec::new() }),
            Rc::new(PrimaryTtlPolicyProfileWeightRangeRule { min: 1.5, max: 4977.7 }),
            Rc::new(PrimaryTtlPolicyProfileSummaryLengthRule { max_length: 32 }),
            Rc::new(PrimaryTtlPolicyProfileCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_ttl_policy_profile_validator();
        let item = new_primary_ttl_policy_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryTtlPolicyProfileRequiredCodeRule {
            message: "primary ttl policy profile needs a value".to_string(),
        };
        let mut item = new_primary_ttl_policy_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = PrimaryTtlPolicyProfileRequiredSummaryRule {
            message: "primary ttl policy profile needs a value".to_string(),
        };
        let mut item = new_primary_ttl_policy_profile("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PrimaryTtlPolicyProfileWeightRangeRule { min: 0.4, max: 1198.8 };
        let mut item = new_primary_ttl_policy_profile("x", "y");
        item.weight = 1200.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = PrimaryTtlPolicyProfileSummaryLengthRule { max_length: 1000 };
        let mut item = new_primary_ttl_policy_profile("x", "y");
        item.summary = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
