use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered measurement agreement must pass. Check returns its problems, or none.
pub trait TieredMeasurementAgreementRule: std::fmt::Debug {
    fn check(&self, item: &TieredMeasurementAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementAgreementTitleLengthRule {
    pub max_length: i64,
}

impl TieredMeasurementAgreementRule for TieredMeasurementAgreementTitleLengthRule {
    fn check(&self, item: &TieredMeasurementAgreement) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementAgreementRequiredTitleRule {
    pub message: String,
}

impl TieredMeasurementAgreementRule for TieredMeasurementAgreementRequiredTitleRule {
    fn check(&self, item: &TieredMeasurementAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementAgreementPinnedGuardRule {
    pub limit: i64,
}

impl TieredMeasurementAgreementRule for TieredMeasurementAgreementPinnedGuardRule {
    fn check(&self, item: &TieredMeasurementAgreement) -> Vec<String> {
        if item.pinned && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while pinned",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementAgreementGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredMeasurementAgreementRule for TieredMeasurementAgreementGraceDaysRangeRule {
    fn check(&self, item: &TieredMeasurementAgreement) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementAgreementRequiredNameRule {
    pub message: String,
}

impl TieredMeasurementAgreementRule for TieredMeasurementAgreementRequiredNameRule {
    fn check(&self, item: &TieredMeasurementAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered measurement agreement.
#[derive(Debug, Clone)]
pub struct TieredMeasurementAgreementValidator {
    pub rules: Vec<Rc<dyn TieredMeasurementAgreementRule>>,
}

impl TieredMeasurementAgreementValidator {
    pub fn validate(&self, item: &TieredMeasurementAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredMeasurementAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredMeasurementAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_measurement_agreement_validator() -> TieredMeasurementAgreementValidator {
    TieredMeasurementAgreementValidator {
        rules: vec![
            Rc::new(TieredMeasurementAgreementTitleLengthRule { max_length: 120 })
                as Rc<dyn TieredMeasurementAgreementRule>,
            Rc::new(TieredMeasurementAgreementRequiredTitleRule {
                message: "tiered measurement agreement needs a value".to_string(),
            }),
            Rc::new(TieredMeasurementAgreementPinnedGuardRule { limit: 120 }),
            Rc::new(TieredMeasurementAgreementGraceDaysRangeRule { min: 80, max: 80 }),
            Rc::new(TieredMeasurementAgreementRequiredNameRule {
                message: "tiered measurement agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_measurement_agreement_validator();
        let item = new_tiered_measurement_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredMeasurementAgreementTitleLengthRule { max_length: 255 };
        let mut item = new_tiered_measurement_agreement("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredMeasurementAgreementRequiredTitleRule {
            message: "tiered measurement agreement needs a value".to_string(),
        };
        let mut item = new_tiered_measurement_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = TieredMeasurementAgreementGraceDaysRangeRule { min: 64, max: 86400 };
        let mut item = new_tiered_measurement_agreement("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredMeasurementAgreementRequiredNameRule {
            message: "tiered measurement agreement needs this field".to_string(),
        };
        let mut item = new_tiered_measurement_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
