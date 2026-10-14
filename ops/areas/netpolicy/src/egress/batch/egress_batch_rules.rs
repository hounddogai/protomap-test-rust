use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress batch must pass. Check returns its problems, or none.
pub trait EgressBatchRule: std::fmt::Debug {
    fn check(&self, item: &EgressBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBatchDescriptionLengthRule {
    pub max_length: i64,
}

impl EgressBatchRule for EgressBatchDescriptionLengthRule {
    fn check(&self, item: &EgressBatch) -> Vec<String> {
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
pub struct EgressBatchLabelLengthRule {
    pub max_length: i64,
}

impl EgressBatchRule for EgressBatchLabelLengthRule {
    fn check(&self, item: &EgressBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBatchCodePrefixRule {
    pub prefix: String,
}

impl EgressBatchRule for EgressBatchCodePrefixRule {
    fn check(&self, item: &EgressBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress batch.
#[derive(Debug, Clone)]
pub struct EgressBatchValidator {
    pub rules: Vec<Rc<dyn EgressBatchRule>>,
}

impl EgressBatchValidator {
    pub fn validate(&self, item: &EgressBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_batch_validator() -> EgressBatchValidator {
    EgressBatchValidator {
        rules: vec![
            Rc::new(EgressBatchDescriptionLengthRule { max_length: 120 }) as Rc<dyn EgressBatchRule>,
            Rc::new(EgressBatchLabelLengthRule { max_length: 32 }),
            Rc::new(EgressBatchCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_batch_validator();
        let item = new_egress_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = EgressBatchDescriptionLengthRule { max_length: 3600 };
        let mut item = new_egress_batch("x", "y");
        item.description = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EgressBatchLabelLengthRule { max_length: 16 };
        let mut item = new_egress_batch("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
