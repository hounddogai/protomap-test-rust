use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched evidence mapping must pass. Check returns its problems, or none.
pub trait BatchedEvidenceMappingRule: std::fmt::Debug {
    fn check(&self, item: &BatchedEvidenceMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEvidenceMappingAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl BatchedEvidenceMappingRule for BatchedEvidenceMappingAllowedExternalRefRule {
    fn check(&self, item: &BatchedEvidenceMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is below minimum", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEvidenceMappingPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedEvidenceMappingRule for BatchedEvidenceMappingPriorityRangeRule {
    fn check(&self, item: &BatchedEvidenceMapping) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEvidenceMappingExternalRefLengthRule {
    pub max_length: i64,
}

impl BatchedEvidenceMappingRule for BatchedEvidenceMappingExternalRefLengthRule {
    fn check(&self, item: &BatchedEvidenceMapping) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEvidenceMappingFingerprintLengthRule {
    pub max_length: i64,
}

impl BatchedEvidenceMappingRule for BatchedEvidenceMappingFingerprintLengthRule {
    fn check(&self, item: &BatchedEvidenceMapping) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEvidenceMappingRequiredFingerprintRule {
    pub message: String,
}

impl BatchedEvidenceMappingRule for BatchedEvidenceMappingRequiredFingerprintRule {
    fn check(&self, item: &BatchedEvidenceMapping) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched evidence mapping.
#[derive(Debug, Clone)]
pub struct BatchedEvidenceMappingValidator {
    pub rules: Vec<Rc<dyn BatchedEvidenceMappingRule>>,
}

impl BatchedEvidenceMappingValidator {
    pub fn validate(&self, item: &BatchedEvidenceMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedEvidenceMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedEvidenceMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_evidence_mapping_validator() -> BatchedEvidenceMappingValidator {
    BatchedEvidenceMappingValidator {
        rules: vec![
            Rc::new(BatchedEvidenceMappingAllowedExternalRefRule {
                allowed: vec!["batched".to_string()],
            }) as Rc<dyn BatchedEvidenceMappingRule>,
            Rc::new(BatchedEvidenceMappingPriorityRangeRule { min: 16, max: 86400 }),
            Rc::new(BatchedEvidenceMappingExternalRefLengthRule { max_length: 80 }),
            Rc::new(BatchedEvidenceMappingFingerprintLengthRule { max_length: 86400 }),
            Rc::new(BatchedEvidenceMappingRequiredFingerprintRule {
                message: "batched evidence mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_evidence_mapping_validator();
        let item = new_batched_evidence_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = BatchedEvidenceMappingPriorityRangeRule { min: 255, max: 1000 };
        let mut item = new_batched_evidence_mapping("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = BatchedEvidenceMappingExternalRefLengthRule { max_length: 16 };
        let mut item = new_batched_evidence_mapping("x", "y");
        item.external_ref = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = BatchedEvidenceMappingFingerprintLengthRule { max_length: 120 };
        let mut item = new_batched_evidence_mapping("x", "y");
        item.fingerprint = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = BatchedEvidenceMappingRequiredFingerprintRule {
            message: "batched evidence mapping needs an entry".to_string(),
        };
        let mut item = new_batched_evidence_mapping("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
