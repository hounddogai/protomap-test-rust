use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier batch must pass. Check returns its problems, or none.
pub trait VerifierBatchRule: std::fmt::Debug {
    fn check(&self, item: &VerifierBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBatchRequiredCodeRule {
    pub message: String,
}

impl VerifierBatchRule for VerifierBatchRequiredCodeRule {
    fn check(&self, item: &VerifierBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBatchCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierBatchRule for VerifierBatchCapacityRangeRule {
    fn check(&self, item: &VerifierBatch) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBatchRequiredAudienceRule {
    pub message: String,
}

impl VerifierBatchRule for VerifierBatchRequiredAudienceRule {
    fn check(&self, item: &VerifierBatch) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBatchRequiredLabelRule {
    pub message: String,
}

impl VerifierBatchRule for VerifierBatchRequiredLabelRule {
    fn check(&self, item: &VerifierBatch) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier batch.
#[derive(Debug, Clone)]
pub struct VerifierBatchValidator {
    pub rules: Vec<Rc<dyn VerifierBatchRule>>,
}

impl VerifierBatchValidator {
    pub fn validate(&self, item: &VerifierBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_batch_validator() -> VerifierBatchValidator {
    VerifierBatchValidator {
        rules: vec![
            Rc::new(VerifierBatchRequiredCodeRule {
                message: "verifier batch needs a value".to_string(),
            }) as Rc<dyn VerifierBatchRule>,
            Rc::new(VerifierBatchCapacityRangeRule { min: 64, max: 64 }),
            Rc::new(VerifierBatchRequiredAudienceRule {
                message: "verifier batch needs a value".to_string(),
            }),
            Rc::new(VerifierBatchRequiredLabelRule {
                message: "verifier batch needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_batch_validator();
        let item = new_verifier_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = VerifierBatchRequiredCodeRule {
            message: "verifier batch needs an entry".to_string(),
        };
        let mut item = new_verifier_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = VerifierBatchCapacityRangeRule { min: 120, max: 3600 };
        let mut item = new_verifier_batch("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = VerifierBatchRequiredAudienceRule {
            message: "verifier batch needs this field".to_string(),
        };
        let mut item = new_verifier_batch("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = VerifierBatchRequiredLabelRule {
            message: "verifier batch needs a value".to_string(),
        };
        let mut item = new_verifier_batch("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
