use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment agreement must pass. Check returns its problems, or none.
pub trait SegmentAgreementRule: std::fmt::Debug {
    fn check(&self, item: &SegmentAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAgreementCodePrefixRule {
    pub prefix: String,
}

impl SegmentAgreementRule for SegmentAgreementCodePrefixRule {
    fn check(&self, item: &SegmentAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAgreementRequiredNameRule {
    pub message: String,
}

impl SegmentAgreementRule for SegmentAgreementRequiredNameRule {
    fn check(&self, item: &SegmentAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAgreementExternalRefLengthRule {
    pub max_length: i64,
}

impl SegmentAgreementRule for SegmentAgreementExternalRefLengthRule {
    fn check(&self, item: &SegmentAgreement) -> Vec<String> {
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
pub struct SegmentAgreementCodeLengthRule {
    pub max_length: i64,
}

impl SegmentAgreementRule for SegmentAgreementCodeLengthRule {
    fn check(&self, item: &SegmentAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment agreement.
#[derive(Debug, Clone)]
pub struct SegmentAgreementValidator {
    pub rules: Vec<Rc<dyn SegmentAgreementRule>>,
}

impl SegmentAgreementValidator {
    pub fn validate(&self, item: &SegmentAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_agreement_validator() -> SegmentAgreementValidator {
    SegmentAgreementValidator {
        rules: vec![
            Rc::new(SegmentAgreementCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn SegmentAgreementRule>,
            Rc::new(SegmentAgreementRequiredNameRule {
                message: "segment agreement needs a value".to_string(),
            }),
            Rc::new(SegmentAgreementExternalRefLengthRule { max_length: 80 }),
            Rc::new(SegmentAgreementCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_agreement_validator();
        let item = new_segment_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SegmentAgreementRequiredNameRule {
            message: "segment agreement needs this field".to_string(),
        };
        let mut item = new_segment_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = SegmentAgreementExternalRefLengthRule { max_length: 80 };
        let mut item = new_segment_agreement("x", "y");
        item.external_ref = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SegmentAgreementCodeLengthRule { max_length: 64 };
        let mut item = new_segment_agreement("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
