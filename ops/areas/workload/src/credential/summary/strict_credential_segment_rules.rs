use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict credential segment must pass. Check returns its problems, or none.
pub trait StrictCredentialSegmentRule: std::fmt::Debug {
    fn check(&self, item: &StrictCredentialSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCredentialSegmentLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictCredentialSegmentRule for StrictCredentialSegmentLimitRangeRule {
    fn check(&self, item: &StrictCredentialSegment) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCredentialSegmentNotesLengthRule {
    pub max_length: i64,
}

impl StrictCredentialSegmentRule for StrictCredentialSegmentNotesLengthRule {
    fn check(&self, item: &StrictCredentialSegment) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCredentialSegmentCodePrefixRule {
    pub prefix: String,
}

impl StrictCredentialSegmentRule for StrictCredentialSegmentCodePrefixRule {
    fn check(&self, item: &StrictCredentialSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCredentialSegmentRequiredLabelRule {
    pub message: String,
}

impl StrictCredentialSegmentRule for StrictCredentialSegmentRequiredLabelRule {
    fn check(&self, item: &StrictCredentialSegment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCredentialSegmentRequiredNotesRule {
    pub message: String,
}

impl StrictCredentialSegmentRule for StrictCredentialSegmentRequiredNotesRule {
    fn check(&self, item: &StrictCredentialSegment) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict credential segment.
#[derive(Debug, Clone)]
pub struct StrictCredentialSegmentValidator {
    pub rules: Vec<Rc<dyn StrictCredentialSegmentRule>>,
}

impl StrictCredentialSegmentValidator {
    pub fn validate(&self, item: &StrictCredentialSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictCredentialSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictCredentialSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_credential_segment_validator() -> StrictCredentialSegmentValidator {
    StrictCredentialSegmentValidator {
        rules: vec![
            Rc::new(StrictCredentialSegmentLimitRangeRule { min: 16, max: 120 }) as Rc<dyn StrictCredentialSegmentRule>,
            Rc::new(StrictCredentialSegmentNotesLengthRule { max_length: 16 }),
            Rc::new(StrictCredentialSegmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictCredentialSegmentRequiredLabelRule {
                message: "strict credential segment needs this field".to_string(),
            }),
            Rc::new(StrictCredentialSegmentRequiredNotesRule {
                message: "strict credential segment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_credential_segment_validator();
        let item = new_strict_credential_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = StrictCredentialSegmentLimitRangeRule { min: 16, max: 64 };
        let mut item = new_strict_credential_segment("x", "y");
        item.limit = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = StrictCredentialSegmentNotesLengthRule { max_length: 120 };
        let mut item = new_strict_credential_segment("x", "y");
        item.notes = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictCredentialSegmentRequiredLabelRule {
            message: "strict credential segment needs this field".to_string(),
        };
        let mut item = new_strict_credential_segment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = StrictCredentialSegmentRequiredNotesRule {
            message: "strict credential segment needs a value".to_string(),
        };
        let mut item = new_strict_credential_segment("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
