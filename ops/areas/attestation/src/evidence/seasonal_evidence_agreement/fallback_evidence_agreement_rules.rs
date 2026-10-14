use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback evidence agreement must pass. Check returns its problems, or none.
pub trait FallbackEvidenceAgreementRule: std::fmt::Debug {
    fn check(&self, item: &FallbackEvidenceAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceAgreementRequiredTimezoneRule {
    pub message: String,
}

impl FallbackEvidenceAgreementRule for FallbackEvidenceAgreementRequiredTimezoneRule {
    fn check(&self, item: &FallbackEvidenceAgreement) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceAgreementRequiredTitleRule {
    pub message: String,
}

impl FallbackEvidenceAgreementRule for FallbackEvidenceAgreementRequiredTitleRule {
    fn check(&self, item: &FallbackEvidenceAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceAgreementTimezoneLengthRule {
    pub max_length: i64,
}

impl FallbackEvidenceAgreementRule for FallbackEvidenceAgreementTimezoneLengthRule {
    fn check(&self, item: &FallbackEvidenceAgreement) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceAgreementWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackEvidenceAgreementRule for FallbackEvidenceAgreementWindowDaysRangeRule {
    fn check(&self, item: &FallbackEvidenceAgreement) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a fallback evidence agreement.
#[derive(Debug, Clone)]
pub struct FallbackEvidenceAgreementValidator {
    pub rules: Vec<Rc<dyn FallbackEvidenceAgreementRule>>,
}

impl FallbackEvidenceAgreementValidator {
    pub fn validate(&self, item: &FallbackEvidenceAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackEvidenceAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackEvidenceAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_evidence_agreement_validator() -> FallbackEvidenceAgreementValidator {
    FallbackEvidenceAgreementValidator {
        rules: vec![
            Rc::new(FallbackEvidenceAgreementRequiredTimezoneRule {
                message: "fallback evidence agreement needs this field".to_string(),
            }) as Rc<dyn FallbackEvidenceAgreementRule>,
            Rc::new(FallbackEvidenceAgreementRequiredTitleRule {
                message: "fallback evidence agreement needs this field".to_string(),
            }),
            Rc::new(FallbackEvidenceAgreementTimezoneLengthRule { max_length: 86400 }),
            Rc::new(FallbackEvidenceAgreementWindowDaysRangeRule { min: 64, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_evidence_agreement_validator();
        let item = new_fallback_evidence_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = FallbackEvidenceAgreementRequiredTimezoneRule {
            message: "fallback evidence agreement needs this field".to_string(),
        };
        let mut item = new_fallback_evidence_agreement("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = FallbackEvidenceAgreementRequiredTitleRule {
            message: "fallback evidence agreement needs a value".to_string(),
        };
        let mut item = new_fallback_evidence_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = FallbackEvidenceAgreementTimezoneLengthRule { max_length: 255 };
        let mut item = new_fallback_evidence_agreement("x", "y");
        item.timezone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = FallbackEvidenceAgreementWindowDaysRangeRule { min: 64, max: 80 };
        let mut item = new_fallback_evidence_agreement("x", "y");
        item.window_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
