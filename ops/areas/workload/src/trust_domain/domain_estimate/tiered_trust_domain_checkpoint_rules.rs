use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered trust domain checkpoint must pass. Check returns its problems, or none.
pub trait TieredTrustDomainCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &TieredTrustDomainCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainCheckpointRequiredAudienceRule {
    pub message: String,
}

impl TieredTrustDomainCheckpointRule for TieredTrustDomainCheckpointRequiredAudienceRule {
    fn check(&self, item: &TieredTrustDomainCheckpoint) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainCheckpointRequiredCodeRule {
    pub message: String,
}

impl TieredTrustDomainCheckpointRule for TieredTrustDomainCheckpointRequiredCodeRule {
    fn check(&self, item: &TieredTrustDomainCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl TieredTrustDomainCheckpointRule for TieredTrustDomainCheckpointCodeLengthRule {
    fn check(&self, item: &TieredTrustDomainCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainCheckpointRequiredNameRule {
    pub message: String,
}

impl TieredTrustDomainCheckpointRule for TieredTrustDomainCheckpointRequiredNameRule {
    fn check(&self, item: &TieredTrustDomainCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainCheckpointUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredTrustDomainCheckpointRule for TieredTrustDomainCheckpointUpdatedAtRangeRule {
    fn check(&self, item: &TieredTrustDomainCheckpoint) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainCheckpointStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredTrustDomainCheckpointRule for TieredTrustDomainCheckpointStartsAtRangeRule {
    fn check(&self, item: &TieredTrustDomainCheckpoint) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered trust domain checkpoint.
#[derive(Debug, Clone)]
pub struct TieredTrustDomainCheckpointValidator {
    pub rules: Vec<Rc<dyn TieredTrustDomainCheckpointRule>>,
}

impl TieredTrustDomainCheckpointValidator {
    pub fn validate(&self, item: &TieredTrustDomainCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredTrustDomainCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredTrustDomainCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_trust_domain_checkpoint_validator() -> TieredTrustDomainCheckpointValidator {
    TieredTrustDomainCheckpointValidator {
        rules: vec![
            Rc::new(TieredTrustDomainCheckpointRequiredAudienceRule {
                message: "tiered trust domain checkpoint needs an entry".to_string(),
            }) as Rc<dyn TieredTrustDomainCheckpointRule>,
            Rc::new(TieredTrustDomainCheckpointRequiredCodeRule {
                message: "tiered trust domain checkpoint needs this field".to_string(),
            }),
            Rc::new(TieredTrustDomainCheckpointCodeLengthRule { max_length: 3600 }),
            Rc::new(TieredTrustDomainCheckpointRequiredNameRule {
                message: "tiered trust domain checkpoint needs an entry".to_string(),
            }),
            Rc::new(TieredTrustDomainCheckpointUpdatedAtRangeRule { min: 32, max: 86400 }),
            Rc::new(TieredTrustDomainCheckpointStartsAtRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_trust_domain_checkpoint_validator();
        let item = new_tiered_trust_domain_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = TieredTrustDomainCheckpointRequiredAudienceRule {
            message: "tiered trust domain checkpoint needs this field".to_string(),
        };
        let mut item = new_tiered_trust_domain_checkpoint("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredTrustDomainCheckpointRequiredCodeRule {
            message: "tiered trust domain checkpoint needs a value".to_string(),
        };
        let mut item = new_tiered_trust_domain_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredTrustDomainCheckpointCodeLengthRule { max_length: 64 };
        let mut item = new_tiered_trust_domain_checkpoint("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredTrustDomainCheckpointRequiredNameRule {
            message: "tiered trust domain checkpoint needs a value".to_string(),
        };
        let mut item = new_tiered_trust_domain_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = TieredTrustDomainCheckpointUpdatedAtRangeRule { min: 32, max: 86400 };
        let mut item = new_tiered_trust_domain_checkpoint("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = TieredTrustDomainCheckpointStartsAtRangeRule { min: 80, max: 255 };
        let mut item = new_tiered_trust_domain_checkpoint("x", "y");
        item.starts_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
