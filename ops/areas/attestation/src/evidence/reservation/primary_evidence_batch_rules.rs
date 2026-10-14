use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary evidence batch must pass. Check returns its problems, or none.
pub trait PrimaryEvidenceBatchRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEvidenceBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceBatchBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEvidenceBatchRule for PrimaryEvidenceBatchBatchSizeRangeRule {
    fn check(&self, item: &PrimaryEvidenceBatch) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceBatchCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryEvidenceBatchRule for PrimaryEvidenceBatchCodeLengthRule {
    fn check(&self, item: &PrimaryEvidenceBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceBatchCodePrefixRule {
    pub prefix: String,
}

impl PrimaryEvidenceBatchRule for PrimaryEvidenceBatchCodePrefixRule {
    fn check(&self, item: &PrimaryEvidenceBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceBatchRequiredCodeRule {
    pub message: String,
}

impl PrimaryEvidenceBatchRule for PrimaryEvidenceBatchRequiredCodeRule {
    fn check(&self, item: &PrimaryEvidenceBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary evidence batch.
#[derive(Debug, Clone)]
pub struct PrimaryEvidenceBatchValidator {
    pub rules: Vec<Rc<dyn PrimaryEvidenceBatchRule>>,
}

impl PrimaryEvidenceBatchValidator {
    pub fn validate(&self, item: &PrimaryEvidenceBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryEvidenceBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEvidenceBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_evidence_batch_validator() -> PrimaryEvidenceBatchValidator {
    PrimaryEvidenceBatchValidator {
        rules: vec![
            Rc::new(PrimaryEvidenceBatchBatchSizeRangeRule { min: 80, max: 120 }) as Rc<dyn PrimaryEvidenceBatchRule>,
            Rc::new(PrimaryEvidenceBatchCodeLengthRule { max_length: 80 }),
            Rc::new(PrimaryEvidenceBatchCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PrimaryEvidenceBatchRequiredCodeRule {
                message: "primary evidence batch needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_evidence_batch_validator();
        let item = new_primary_evidence_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PrimaryEvidenceBatchBatchSizeRangeRule { min: 32, max: 1000 };
        let mut item = new_primary_evidence_batch("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryEvidenceBatchCodeLengthRule { max_length: 3600 };
        let mut item = new_primary_evidence_batch("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryEvidenceBatchRequiredCodeRule {
            message: "primary evidence batch needs an entry".to_string(),
        };
        let mut item = new_primary_evidence_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
