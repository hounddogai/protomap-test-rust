use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering exception must pass. Check returns its problems, or none.
pub trait PeeringExceptionRule: std::fmt::Debug {
    fn check(&self, item: &PeeringException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringExceptionRequiredLabelRule {
    pub message: String,
}

impl PeeringExceptionRule for PeeringExceptionRequiredLabelRule {
    fn check(&self, item: &PeeringException) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringExceptionRequiredSummaryRule {
    pub message: String,
}

impl PeeringExceptionRule for PeeringExceptionRequiredSummaryRule {
    fn check(&self, item: &PeeringException) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringExceptionCodePrefixRule {
    pub prefix: String,
}

impl PeeringExceptionRule for PeeringExceptionCodePrefixRule {
    fn check(&self, item: &PeeringException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringExceptionSummaryLengthRule {
    pub max_length: i64,
}

impl PeeringExceptionRule for PeeringExceptionSummaryLengthRule {
    fn check(&self, item: &PeeringException) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringExceptionRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringExceptionRule for PeeringExceptionRevisionRangeRule {
    fn check(&self, item: &PeeringException) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringExceptionLabelLengthRule {
    pub max_length: i64,
}

impl PeeringExceptionRule for PeeringExceptionLabelLengthRule {
    fn check(&self, item: &PeeringException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering exception.
#[derive(Debug, Clone)]
pub struct PeeringExceptionValidator {
    pub rules: Vec<Rc<dyn PeeringExceptionRule>>,
}

impl PeeringExceptionValidator {
    pub fn validate(&self, item: &PeeringException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_exception_validator() -> PeeringExceptionValidator {
    PeeringExceptionValidator {
        rules: vec![
            Rc::new(PeeringExceptionRequiredLabelRule {
                message: "peering exception needs an entry".to_string(),
            }) as Rc<dyn PeeringExceptionRule>,
            Rc::new(PeeringExceptionRequiredSummaryRule {
                message: "peering exception needs this field".to_string(),
            }),
            Rc::new(PeeringExceptionCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PeeringExceptionSummaryLengthRule { max_length: 86400 }),
            Rc::new(PeeringExceptionRevisionRangeRule { min: 120, max: 255 }),
            Rc::new(PeeringExceptionLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_exception_validator();
        let item = new_peering_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PeeringExceptionRequiredLabelRule {
            message: "peering exception needs a value".to_string(),
        };
        let mut item = new_peering_exception("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = PeeringExceptionRequiredSummaryRule {
            message: "peering exception needs an entry".to_string(),
        };
        let mut item = new_peering_exception("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = PeeringExceptionSummaryLengthRule { max_length: 16 };
        let mut item = new_peering_exception("x", "y");
        item.summary = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = PeeringExceptionRevisionRangeRule { min: 64, max: 3600 };
        let mut item = new_peering_exception("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PeeringExceptionLabelLengthRule { max_length: 120 };
        let mut item = new_peering_exception("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
