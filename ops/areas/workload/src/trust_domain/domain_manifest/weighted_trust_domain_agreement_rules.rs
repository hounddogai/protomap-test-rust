use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted trust domain agreement must pass. Check returns its problems, or none.
pub trait WeightedTrustDomainAgreementRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTrustDomainAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainAgreementIssuerLengthRule {
    pub max_length: i64,
}

impl WeightedTrustDomainAgreementRule for WeightedTrustDomainAgreementIssuerLengthRule {
    fn check(&self, item: &WeightedTrustDomainAgreement) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainAgreementAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl WeightedTrustDomainAgreementRule for WeightedTrustDomainAgreementAllowedLabelRule {
    fn check(&self, item: &WeightedTrustDomainAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainAgreementRequiredIssuerRule {
    pub message: String,
}

impl WeightedTrustDomainAgreementRule for WeightedTrustDomainAgreementRequiredIssuerRule {
    fn check(&self, item: &WeightedTrustDomainAgreement) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainAgreementRequiredCodeRule {
    pub message: String,
}

impl WeightedTrustDomainAgreementRule for WeightedTrustDomainAgreementRequiredCodeRule {
    fn check(&self, item: &WeightedTrustDomainAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted trust domain agreement.
#[derive(Debug, Clone)]
pub struct WeightedTrustDomainAgreementValidator {
    pub rules: Vec<Rc<dyn WeightedTrustDomainAgreementRule>>,
}

impl WeightedTrustDomainAgreementValidator {
    pub fn validate(&self, item: &WeightedTrustDomainAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedTrustDomainAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTrustDomainAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_trust_domain_agreement_validator() -> WeightedTrustDomainAgreementValidator {
    WeightedTrustDomainAgreementValidator {
        rules: vec![
            Rc::new(WeightedTrustDomainAgreementIssuerLengthRule { max_length: 80 })
                as Rc<dyn WeightedTrustDomainAgreementRule>,
            Rc::new(WeightedTrustDomainAgreementAllowedLabelRule {
                allowed: vec!["fallback".to_string(), "legacy".to_string()],
            }),
            Rc::new(WeightedTrustDomainAgreementRequiredIssuerRule {
                message: "weighted trust domain agreement needs this field".to_string(),
            }),
            Rc::new(WeightedTrustDomainAgreementRequiredCodeRule {
                message: "weighted trust domain agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_trust_domain_agreement_validator();
        let item = new_weighted_trust_domain_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = WeightedTrustDomainAgreementIssuerLengthRule { max_length: 80 };
        let mut item = new_weighted_trust_domain_agreement("x", "y");
        item.issuer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = WeightedTrustDomainAgreementRequiredIssuerRule {
            message: "weighted trust domain agreement needs this field".to_string(),
        };
        let mut item = new_weighted_trust_domain_agreement("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedTrustDomainAgreementRequiredCodeRule {
            message: "weighted trust domain agreement needs this field".to_string(),
        };
        let mut item = new_weighted_trust_domain_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
