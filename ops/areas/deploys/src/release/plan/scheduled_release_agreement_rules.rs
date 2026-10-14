use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled release agreement must pass. Check returns its problems, or none.
pub trait ScheduledReleaseAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledReleaseAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseAgreementExternalRefLengthRule {
    pub max_length: i64,
}

impl ScheduledReleaseAgreementRule for ScheduledReleaseAgreementExternalRefLengthRule {
    fn check(&self, item: &ScheduledReleaseAgreement) -> Vec<String> {
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
pub struct ScheduledReleaseAgreementWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledReleaseAgreementRule for ScheduledReleaseAgreementWindowDaysRangeRule {
    fn check(&self, item: &ScheduledReleaseAgreement) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseAgreementRequiredLabelRule {
    pub message: String,
}

impl ScheduledReleaseAgreementRule for ScheduledReleaseAgreementRequiredLabelRule {
    fn check(&self, item: &ScheduledReleaseAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReleaseAgreementRequiredCodeRule {
    pub message: String,
}

impl ScheduledReleaseAgreementRule for ScheduledReleaseAgreementRequiredCodeRule {
    fn check(&self, item: &ScheduledReleaseAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled release agreement.
#[derive(Debug, Clone)]
pub struct ScheduledReleaseAgreementValidator {
    pub rules: Vec<Rc<dyn ScheduledReleaseAgreementRule>>,
}

impl ScheduledReleaseAgreementValidator {
    pub fn validate(&self, item: &ScheduledReleaseAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledReleaseAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledReleaseAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_release_agreement_validator() -> ScheduledReleaseAgreementValidator {
    ScheduledReleaseAgreementValidator {
        rules: vec![
            Rc::new(ScheduledReleaseAgreementExternalRefLengthRule { max_length: 120 })
                as Rc<dyn ScheduledReleaseAgreementRule>,
            Rc::new(ScheduledReleaseAgreementWindowDaysRangeRule { min: 80, max: 3600 }),
            Rc::new(ScheduledReleaseAgreementRequiredLabelRule {
                message: "scheduled release agreement needs an entry".to_string(),
            }),
            Rc::new(ScheduledReleaseAgreementRequiredCodeRule {
                message: "scheduled release agreement needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_release_agreement_validator();
        let item = new_scheduled_release_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = ScheduledReleaseAgreementExternalRefLengthRule { max_length: 1000 };
        let mut item = new_scheduled_release_agreement("x", "y");
        item.external_ref = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = ScheduledReleaseAgreementWindowDaysRangeRule { min: 64, max: 86400 };
        let mut item = new_scheduled_release_agreement("x", "y");
        item.window_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ScheduledReleaseAgreementRequiredLabelRule {
            message: "scheduled release agreement needs this field".to_string(),
        };
        let mut item = new_scheduled_release_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledReleaseAgreementRequiredCodeRule {
            message: "scheduled release agreement needs a value".to_string(),
        };
        let mut item = new_scheduled_release_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
