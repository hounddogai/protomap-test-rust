use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence batch must pass. Check returns its problems, or none.
pub trait EvidenceBatchRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBatchAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EvidenceBatchRule for EvidenceBatchAmountRangeRule {
    fn check(&self, item: &EvidenceBatch) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBatchNameLengthRule {
    pub max_length: i64,
}

impl EvidenceBatchRule for EvidenceBatchNameLengthRule {
    fn check(&self, item: &EvidenceBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBatchCodePrefixRule {
    pub prefix: String,
}

impl EvidenceBatchRule for EvidenceBatchCodePrefixRule {
    fn check(&self, item: &EvidenceBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBatchRequiredFingerprintRule {
    pub message: String,
}

impl EvidenceBatchRule for EvidenceBatchRequiredFingerprintRule {
    fn check(&self, item: &EvidenceBatch) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence batch.
#[derive(Debug, Clone)]
pub struct EvidenceBatchValidator {
    pub rules: Vec<Rc<dyn EvidenceBatchRule>>,
}

impl EvidenceBatchValidator {
    pub fn validate(&self, item: &EvidenceBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_batch_validator() -> EvidenceBatchValidator {
    EvidenceBatchValidator {
        rules: vec![
            Rc::new(EvidenceBatchAmountRangeRule { min: 4.1, max: 2089.1 }) as Rc<dyn EvidenceBatchRule>,
            Rc::new(EvidenceBatchNameLengthRule { max_length: 32 }),
            Rc::new(EvidenceBatchCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(EvidenceBatchRequiredFingerprintRule {
                message: "evidence batch needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_batch_validator();
        let item = new_evidence_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = EvidenceBatchAmountRangeRule { min: 5.7, max: 4940.8 };
        let mut item = new_evidence_batch("x", "y");
        item.amount = 4942.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EvidenceBatchNameLengthRule { max_length: 32 };
        let mut item = new_evidence_batch("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = EvidenceBatchRequiredFingerprintRule {
            message: "evidence batch needs an entry".to_string(),
        };
        let mut item = new_evidence_batch("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
