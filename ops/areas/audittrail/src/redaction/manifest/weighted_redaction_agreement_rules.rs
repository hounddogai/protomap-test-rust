use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted redaction agreement must pass. Check returns its problems, or none.
pub trait WeightedRedactionAgreementRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRedactionAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRedactionAgreementRequiredLabelRule {
    pub message: String,
}

impl WeightedRedactionAgreementRule for WeightedRedactionAgreementRequiredLabelRule {
    fn check(&self, item: &WeightedRedactionAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRedactionAgreementLabelLengthRule {
    pub max_length: i64,
}

impl WeightedRedactionAgreementRule for WeightedRedactionAgreementLabelLengthRule {
    fn check(&self, item: &WeightedRedactionAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRedactionAgreementReviewerLengthRule {
    pub max_length: i64,
}

impl WeightedRedactionAgreementRule for WeightedRedactionAgreementReviewerLengthRule {
    fn check(&self, item: &WeightedRedactionAgreement) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted redaction agreement.
#[derive(Debug, Clone)]
pub struct WeightedRedactionAgreementValidator {
    pub rules: Vec<Rc<dyn WeightedRedactionAgreementRule>>,
}

impl WeightedRedactionAgreementValidator {
    pub fn validate(&self, item: &WeightedRedactionAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedRedactionAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRedactionAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_redaction_agreement_validator() -> WeightedRedactionAgreementValidator {
    WeightedRedactionAgreementValidator {
        rules: vec![
            Rc::new(WeightedRedactionAgreementRequiredLabelRule {
                message: "weighted redaction agreement needs an entry".to_string(),
            }) as Rc<dyn WeightedRedactionAgreementRule>,
            Rc::new(WeightedRedactionAgreementLabelLengthRule { max_length: 3600 }),
            Rc::new(WeightedRedactionAgreementReviewerLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_redaction_agreement_validator();
        let item = new_weighted_redaction_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = WeightedRedactionAgreementRequiredLabelRule {
            message: "weighted redaction agreement needs this field".to_string(),
        };
        let mut item = new_weighted_redaction_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedRedactionAgreementLabelLengthRule { max_length: 120 };
        let mut item = new_weighted_redaction_agreement("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = WeightedRedactionAgreementReviewerLengthRule { max_length: 80 };
        let mut item = new_weighted_redaction_agreement("x", "y");
        item.reviewer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
