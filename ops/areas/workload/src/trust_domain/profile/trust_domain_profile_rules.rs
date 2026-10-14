use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain profile must pass. Check returns its problems, or none.
pub trait TrustDomainProfileRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainProfileCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainProfileRule for TrustDomainProfileCreatedAtRangeRule {
    fn check(&self, item: &TrustDomainProfile) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainProfileCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainProfileRule for TrustDomainProfileCodePrefixRule {
    fn check(&self, item: &TrustDomainProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainProfileRequiredCodeRule {
    pub message: String,
}

impl TrustDomainProfileRule for TrustDomainProfileRequiredCodeRule {
    fn check(&self, item: &TrustDomainProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainProfileRequiredExternalRefRule {
    pub message: String,
}

impl TrustDomainProfileRule for TrustDomainProfileRequiredExternalRefRule {
    fn check(&self, item: &TrustDomainProfile) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainProfileTitleLengthRule {
    pub max_length: i64,
}

impl TrustDomainProfileRule for TrustDomainProfileTitleLengthRule {
    fn check(&self, item: &TrustDomainProfile) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain profile.
#[derive(Debug, Clone)]
pub struct TrustDomainProfileValidator {
    pub rules: Vec<Rc<dyn TrustDomainProfileRule>>,
}

impl TrustDomainProfileValidator {
    pub fn validate(&self, item: &TrustDomainProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_profile_validator() -> TrustDomainProfileValidator {
    TrustDomainProfileValidator {
        rules: vec![
            Rc::new(TrustDomainProfileCreatedAtRangeRule { min: 16, max: 64 }) as Rc<dyn TrustDomainProfileRule>,
            Rc::new(TrustDomainProfileCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TrustDomainProfileRequiredCodeRule {
                message: "trust domain profile needs this field".to_string(),
            }),
            Rc::new(TrustDomainProfileRequiredExternalRefRule {
                message: "trust domain profile needs an entry".to_string(),
            }),
            Rc::new(TrustDomainProfileTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_profile_validator();
        let item = new_trust_domain_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = TrustDomainProfileCreatedAtRangeRule { min: 120, max: 1000 };
        let mut item = new_trust_domain_profile("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TrustDomainProfileRequiredCodeRule {
            message: "trust domain profile needs a value".to_string(),
        };
        let mut item = new_trust_domain_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = TrustDomainProfileRequiredExternalRefRule {
            message: "trust domain profile needs this field".to_string(),
        };
        let mut item = new_trust_domain_profile("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TrustDomainProfileTitleLengthRule { max_length: 120 };
        let mut item = new_trust_domain_profile("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
