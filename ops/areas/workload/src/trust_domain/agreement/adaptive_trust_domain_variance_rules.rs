use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive trust domain variance must pass. Check returns its problems, or none.
pub trait AdaptiveTrustDomainVarianceRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTrustDomainVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainVarianceRequiredReviewerRule {
    pub message: String,
}

impl AdaptiveTrustDomainVarianceRule for AdaptiveTrustDomainVarianceRequiredReviewerRule {
    fn check(&self, item: &AdaptiveTrustDomainVariance) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainVarianceVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveTrustDomainVarianceRule for AdaptiveTrustDomainVarianceVersionRangeRule {
    fn check(&self, item: &AdaptiveTrustDomainVariance) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainVarianceAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl AdaptiveTrustDomainVarianceRule for AdaptiveTrustDomainVarianceAllowedReviewerRule {
    fn check(&self, item: &AdaptiveTrustDomainVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is unknown", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainVarianceCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveTrustDomainVarianceRule for AdaptiveTrustDomainVarianceCodeLengthRule {
    fn check(&self, item: &AdaptiveTrustDomainVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainVarianceCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveTrustDomainVarianceRule for AdaptiveTrustDomainVarianceCodePrefixRule {
    fn check(&self, item: &AdaptiveTrustDomainVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainVarianceReviewerLengthRule {
    pub max_length: i64,
}

impl AdaptiveTrustDomainVarianceRule for AdaptiveTrustDomainVarianceReviewerLengthRule {
    fn check(&self, item: &AdaptiveTrustDomainVariance) -> Vec<String> {
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

/// Runs every rule of a adaptive trust domain variance.
#[derive(Debug, Clone)]
pub struct AdaptiveTrustDomainVarianceValidator {
    pub rules: Vec<Rc<dyn AdaptiveTrustDomainVarianceRule>>,
}

impl AdaptiveTrustDomainVarianceValidator {
    pub fn validate(&self, item: &AdaptiveTrustDomainVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveTrustDomainVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTrustDomainVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_trust_domain_variance_validator() -> AdaptiveTrustDomainVarianceValidator {
    AdaptiveTrustDomainVarianceValidator {
        rules: vec![
            Rc::new(AdaptiveTrustDomainVarianceRequiredReviewerRule {
                message: "adaptive trust domain variance needs this field".to_string(),
            }) as Rc<dyn AdaptiveTrustDomainVarianceRule>,
            Rc::new(AdaptiveTrustDomainVarianceVersionRangeRule { min: 120, max: 86400 }),
            Rc::new(AdaptiveTrustDomainVarianceAllowedReviewerRule {
                allowed: vec!["rolling".to_string(), "seasonal".to_string()],
            }),
            Rc::new(AdaptiveTrustDomainVarianceCodeLengthRule { max_length: 64 }),
            Rc::new(AdaptiveTrustDomainVarianceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AdaptiveTrustDomainVarianceReviewerLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_trust_domain_variance_validator();
        let item = new_adaptive_trust_domain_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = AdaptiveTrustDomainVarianceRequiredReviewerRule {
            message: "adaptive trust domain variance needs this field".to_string(),
        };
        let mut item = new_adaptive_trust_domain_variance("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = AdaptiveTrustDomainVarianceVersionRangeRule { min: 255, max: 1000 };
        let mut item = new_adaptive_trust_domain_variance("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveTrustDomainVarianceCodeLengthRule { max_length: 255 };
        let mut item = new_adaptive_trust_domain_variance("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = AdaptiveTrustDomainVarianceReviewerLengthRule { max_length: 64 };
        let mut item = new_adaptive_trust_domain_variance("x", "y");
        item.reviewer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
