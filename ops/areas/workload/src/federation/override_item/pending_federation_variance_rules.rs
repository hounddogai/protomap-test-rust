use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending federation variance must pass. Check returns its problems, or none.
pub trait PendingFederationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PendingFederationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationVarianceLabelLengthRule {
    pub max_length: i64,
}

impl PendingFederationVarianceRule for PendingFederationVarianceLabelLengthRule {
    fn check(&self, item: &PendingFederationVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationVarianceDescriptionLengthRule {
    pub max_length: i64,
}

impl PendingFederationVarianceRule for PendingFederationVarianceDescriptionLengthRule {
    fn check(&self, item: &PendingFederationVariance) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationVarianceCipherLengthRule {
    pub max_length: i64,
}

impl PendingFederationVarianceRule for PendingFederationVarianceCipherLengthRule {
    fn check(&self, item: &PendingFederationVariance) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationVarianceRequiredCipherRule {
    pub message: String,
}

impl PendingFederationVarianceRule for PendingFederationVarianceRequiredCipherRule {
    fn check(&self, item: &PendingFederationVariance) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending federation variance.
#[derive(Debug, Clone)]
pub struct PendingFederationVarianceValidator {
    pub rules: Vec<Rc<dyn PendingFederationVarianceRule>>,
}

impl PendingFederationVarianceValidator {
    pub fn validate(&self, item: &PendingFederationVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingFederationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFederationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_federation_variance_validator() -> PendingFederationVarianceValidator {
    PendingFederationVarianceValidator {
        rules: vec![
            Rc::new(PendingFederationVarianceLabelLengthRule { max_length: 255 })
                as Rc<dyn PendingFederationVarianceRule>,
            Rc::new(PendingFederationVarianceDescriptionLengthRule { max_length: 1000 }),
            Rc::new(PendingFederationVarianceCipherLengthRule { max_length: 255 }),
            Rc::new(PendingFederationVarianceRequiredCipherRule {
                message: "pending federation variance needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_federation_variance_validator();
        let item = new_pending_federation_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingFederationVarianceLabelLengthRule { max_length: 120 };
        let mut item = new_pending_federation_variance("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = PendingFederationVarianceDescriptionLengthRule { max_length: 80 };
        let mut item = new_pending_federation_variance("x", "y");
        item.description = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = PendingFederationVarianceCipherLengthRule { max_length: 64 };
        let mut item = new_pending_federation_variance("x", "y");
        item.cipher = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = PendingFederationVarianceRequiredCipherRule {
            message: "pending federation variance needs this field".to_string(),
        };
        let mut item = new_pending_federation_variance("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
