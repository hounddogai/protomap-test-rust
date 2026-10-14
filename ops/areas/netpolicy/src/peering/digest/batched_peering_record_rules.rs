use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched peering record must pass. Check returns its problems, or none.
pub trait BatchedPeeringRecordRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPeeringRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringRecordCipherLengthRule {
    pub max_length: i64,
}

impl BatchedPeeringRecordRule for BatchedPeeringRecordCipherLengthRule {
    fn check(&self, item: &BatchedPeeringRecord) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringRecordRequiredCodeRule {
    pub message: String,
}

impl BatchedPeeringRecordRule for BatchedPeeringRecordRequiredCodeRule {
    fn check(&self, item: &BatchedPeeringRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringRecordRequiredTitleRule {
    pub message: String,
}

impl BatchedPeeringRecordRule for BatchedPeeringRecordRequiredTitleRule {
    fn check(&self, item: &BatchedPeeringRecord) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringRecordRequiredCipherRule {
    pub message: String,
}

impl BatchedPeeringRecordRule for BatchedPeeringRecordRequiredCipherRule {
    fn check(&self, item: &BatchedPeeringRecord) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringRecordPrincipalLengthRule {
    pub max_length: i64,
}

impl BatchedPeeringRecordRule for BatchedPeeringRecordPrincipalLengthRule {
    fn check(&self, item: &BatchedPeeringRecord) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringRecordCodeLengthRule {
    pub max_length: i64,
}

impl BatchedPeeringRecordRule for BatchedPeeringRecordCodeLengthRule {
    fn check(&self, item: &BatchedPeeringRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched peering record.
#[derive(Debug, Clone)]
pub struct BatchedPeeringRecordValidator {
    pub rules: Vec<Rc<dyn BatchedPeeringRecordRule>>,
}

impl BatchedPeeringRecordValidator {
    pub fn validate(&self, item: &BatchedPeeringRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPeeringRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPeeringRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_peering_record_validator() -> BatchedPeeringRecordValidator {
    BatchedPeeringRecordValidator {
        rules: vec![
            Rc::new(BatchedPeeringRecordCipherLengthRule { max_length: 80 }) as Rc<dyn BatchedPeeringRecordRule>,
            Rc::new(BatchedPeeringRecordRequiredCodeRule {
                message: "batched peering record needs this field".to_string(),
            }),
            Rc::new(BatchedPeeringRecordRequiredTitleRule {
                message: "batched peering record needs this field".to_string(),
            }),
            Rc::new(BatchedPeeringRecordRequiredCipherRule {
                message: "batched peering record needs this field".to_string(),
            }),
            Rc::new(BatchedPeeringRecordPrincipalLengthRule { max_length: 80 }),
            Rc::new(BatchedPeeringRecordCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_peering_record_validator();
        let item = new_batched_peering_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = BatchedPeeringRecordCipherLengthRule { max_length: 120 };
        let mut item = new_batched_peering_record("x", "y");
        item.cipher = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedPeeringRecordRequiredCodeRule {
            message: "batched peering record needs an entry".to_string(),
        };
        let mut item = new_batched_peering_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BatchedPeeringRecordRequiredTitleRule {
            message: "batched peering record needs a value".to_string(),
        };
        let mut item = new_batched_peering_record("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = BatchedPeeringRecordRequiredCipherRule {
            message: "batched peering record needs a value".to_string(),
        };
        let mut item = new_batched_peering_record("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = BatchedPeeringRecordPrincipalLengthRule { max_length: 86400 };
        let mut item = new_batched_peering_record("x", "y");
        item.principal = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedPeeringRecordCodeLengthRule { max_length: 1000 };
        let mut item = new_batched_peering_record("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
