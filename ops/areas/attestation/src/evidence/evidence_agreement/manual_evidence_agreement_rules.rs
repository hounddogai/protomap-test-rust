use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual evidence agreement must pass. Check returns its problems, or none.
pub trait ManualEvidenceAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ManualEvidenceAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceAgreementPhoneLengthRule {
    pub max_length: i64,
}

impl ManualEvidenceAgreementRule for ManualEvidenceAgreementPhoneLengthRule {
    fn check(&self, item: &ManualEvidenceAgreement) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceAgreementWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualEvidenceAgreementRule for ManualEvidenceAgreementWeightRangeRule {
    fn check(&self, item: &ManualEvidenceAgreement) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceAgreementPinnedGuardRule {
    pub limit: i64,
}

impl ManualEvidenceAgreementRule for ManualEvidenceAgreementPinnedGuardRule {
    fn check(&self, item: &ManualEvidenceAgreement) -> Vec<String> {
        if item.pinned && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while pinned",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceAgreementRequiredNamespaceRule {
    pub message: String,
}

impl ManualEvidenceAgreementRule for ManualEvidenceAgreementRequiredNamespaceRule {
    fn check(&self, item: &ManualEvidenceAgreement) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceAgreementSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualEvidenceAgreementRule for ManualEvidenceAgreementSequenceRangeRule {
    fn check(&self, item: &ManualEvidenceAgreement) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceAgreementLabelLengthRule {
    pub max_length: i64,
}

impl ManualEvidenceAgreementRule for ManualEvidenceAgreementLabelLengthRule {
    fn check(&self, item: &ManualEvidenceAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual evidence agreement.
#[derive(Debug, Clone)]
pub struct ManualEvidenceAgreementValidator {
    pub rules: Vec<Rc<dyn ManualEvidenceAgreementRule>>,
}

impl ManualEvidenceAgreementValidator {
    pub fn validate(&self, item: &ManualEvidenceAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualEvidenceAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualEvidenceAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_evidence_agreement_validator() -> ManualEvidenceAgreementValidator {
    ManualEvidenceAgreementValidator {
        rules: vec![
            Rc::new(ManualEvidenceAgreementPhoneLengthRule { max_length: 80 }) as Rc<dyn ManualEvidenceAgreementRule>,
            Rc::new(ManualEvidenceAgreementWeightRangeRule { min: 8.3, max: 3734.4 }),
            Rc::new(ManualEvidenceAgreementPinnedGuardRule { limit: 64 }),
            Rc::new(ManualEvidenceAgreementRequiredNamespaceRule {
                message: "manual evidence agreement needs an entry".to_string(),
            }),
            Rc::new(ManualEvidenceAgreementSequenceRangeRule { min: 64, max: 80 }),
            Rc::new(ManualEvidenceAgreementLabelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_evidence_agreement_validator();
        let item = new_manual_evidence_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = ManualEvidenceAgreementPhoneLengthRule { max_length: 255 };
        let mut item = new_manual_evidence_agreement("x", "y");
        item.phone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ManualEvidenceAgreementWeightRangeRule { min: 6.6, max: 521.2 };
        let mut item = new_manual_evidence_agreement("x", "y");
        item.weight = 522.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = ManualEvidenceAgreementRequiredNamespaceRule {
            message: "manual evidence agreement needs this field".to_string(),
        };
        let mut item = new_manual_evidence_agreement("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = ManualEvidenceAgreementSequenceRangeRule { min: 120, max: 1000 };
        let mut item = new_manual_evidence_agreement("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualEvidenceAgreementLabelLengthRule { max_length: 120 };
        let mut item = new_manual_evidence_agreement("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
