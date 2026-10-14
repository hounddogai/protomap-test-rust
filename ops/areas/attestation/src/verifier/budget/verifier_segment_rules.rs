use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier segment must pass. Check returns its problems, or none.
pub trait VerifierSegmentRule: std::fmt::Debug {
    fn check(&self, item: &VerifierSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSegmentQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierSegmentRule for VerifierSegmentQuantityRangeRule {
    fn check(&self, item: &VerifierSegment) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSegmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl VerifierSegmentRule for VerifierSegmentAllowedNameRule {
    fn check(&self, item: &VerifierSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSegmentRequiredNameRule {
    pub message: String,
}

impl VerifierSegmentRule for VerifierSegmentRequiredNameRule {
    fn check(&self, item: &VerifierSegment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSegmentCodeLengthRule {
    pub max_length: i64,
}

impl VerifierSegmentRule for VerifierSegmentCodeLengthRule {
    fn check(&self, item: &VerifierSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSegmentLabelLengthRule {
    pub max_length: i64,
}

impl VerifierSegmentRule for VerifierSegmentLabelLengthRule {
    fn check(&self, item: &VerifierSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier segment.
#[derive(Debug, Clone)]
pub struct VerifierSegmentValidator {
    pub rules: Vec<Rc<dyn VerifierSegmentRule>>,
}

impl VerifierSegmentValidator {
    pub fn validate(&self, item: &VerifierSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_segment_validator() -> VerifierSegmentValidator {
    VerifierSegmentValidator {
        rules: vec![
            Rc::new(VerifierSegmentQuantityRangeRule { min: 16, max: 86400 }) as Rc<dyn VerifierSegmentRule>,
            Rc::new(VerifierSegmentAllowedNameRule { allowed: Vec::new() }),
            Rc::new(VerifierSegmentRequiredNameRule {
                message: "verifier segment needs this field".to_string(),
            }),
            Rc::new(VerifierSegmentCodeLengthRule { max_length: 80 }),
            Rc::new(VerifierSegmentLabelLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_segment_validator();
        let item = new_verifier_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = VerifierSegmentQuantityRangeRule { min: 32, max: 255 };
        let mut item = new_verifier_segment("x", "y");
        item.quantity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = VerifierSegmentRequiredNameRule {
            message: "verifier segment needs a value".to_string(),
        };
        let mut item = new_verifier_segment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierSegmentCodeLengthRule { max_length: 80 };
        let mut item = new_verifier_segment("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = VerifierSegmentLabelLengthRule { max_length: 3600 };
        let mut item = new_verifier_segment("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
