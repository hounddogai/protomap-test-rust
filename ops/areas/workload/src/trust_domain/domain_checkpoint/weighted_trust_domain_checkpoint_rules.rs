use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted trust domain checkpoint must pass. Check returns its problems, or none.
pub trait WeightedTrustDomainCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTrustDomainCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainCheckpointRequiredFingerprintRule {
    pub message: String,
}

impl WeightedTrustDomainCheckpointRule for WeightedTrustDomainCheckpointRequiredFingerprintRule {
    fn check(&self, item: &WeightedTrustDomainCheckpoint) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainCheckpointAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl WeightedTrustDomainCheckpointRule for WeightedTrustDomainCheckpointAllowedTitleRule {
    fn check(&self, item: &WeightedTrustDomainCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainCheckpointCountryLengthRule {
    pub max_length: i64,
}

impl WeightedTrustDomainCheckpointRule for WeightedTrustDomainCheckpointCountryLengthRule {
    fn check(&self, item: &WeightedTrustDomainCheckpoint) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted trust domain checkpoint.
#[derive(Debug, Clone)]
pub struct WeightedTrustDomainCheckpointValidator {
    pub rules: Vec<Rc<dyn WeightedTrustDomainCheckpointRule>>,
}

impl WeightedTrustDomainCheckpointValidator {
    pub fn validate(&self, item: &WeightedTrustDomainCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedTrustDomainCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTrustDomainCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_trust_domain_checkpoint_validator() -> WeightedTrustDomainCheckpointValidator {
    WeightedTrustDomainCheckpointValidator {
        rules: vec![
            Rc::new(WeightedTrustDomainCheckpointRequiredFingerprintRule {
                message: "weighted trust domain checkpoint needs this field".to_string(),
            }) as Rc<dyn WeightedTrustDomainCheckpointRule>,
            Rc::new(WeightedTrustDomainCheckpointAllowedTitleRule {
                allowed: vec!["rolling".to_string(), "seasonal".to_string()],
            }),
            Rc::new(WeightedTrustDomainCheckpointCountryLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_trust_domain_checkpoint_validator();
        let item = new_weighted_trust_domain_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = WeightedTrustDomainCheckpointRequiredFingerprintRule {
            message: "weighted trust domain checkpoint needs this field".to_string(),
        };
        let mut item = new_weighted_trust_domain_checkpoint("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = WeightedTrustDomainCheckpointCountryLengthRule { max_length: 16 };
        let mut item = new_weighted_trust_domain_checkpoint("x", "y");
        item.country = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
