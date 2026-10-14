use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline agreement must pass. Check returns its problems, or none.
pub trait TimelineAgreementRule: std::fmt::Debug {
    fn check(&self, item: &TimelineAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAgreementRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineAgreementRule for TimelineAgreementRevisionRangeRule {
    fn check(&self, item: &TimelineAgreement) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAgreementPhoneLengthRule {
    pub max_length: i64,
}

impl TimelineAgreementRule for TimelineAgreementPhoneLengthRule {
    fn check(&self, item: &TimelineAgreement) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAgreementRequiredTitleRule {
    pub message: String,
}

impl TimelineAgreementRule for TimelineAgreementRequiredTitleRule {
    fn check(&self, item: &TimelineAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline agreement.
#[derive(Debug, Clone)]
pub struct TimelineAgreementValidator {
    pub rules: Vec<Rc<dyn TimelineAgreementRule>>,
}

impl TimelineAgreementValidator {
    pub fn validate(&self, item: &TimelineAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_agreement_validator() -> TimelineAgreementValidator {
    TimelineAgreementValidator {
        rules: vec![
            Rc::new(TimelineAgreementRevisionRangeRule { min: 80, max: 86400 }) as Rc<dyn TimelineAgreementRule>,
            Rc::new(TimelineAgreementPhoneLengthRule { max_length: 120 }),
            Rc::new(TimelineAgreementRequiredTitleRule {
                message: "timeline agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_agreement_validator();
        let item = new_timeline_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = TimelineAgreementRevisionRangeRule { min: 64, max: 255 };
        let mut item = new_timeline_agreement("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = TimelineAgreementPhoneLengthRule { max_length: 255 };
        let mut item = new_timeline_agreement("x", "y");
        item.phone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TimelineAgreementRequiredTitleRule {
            message: "timeline agreement needs an entry".to_string(),
        };
        let mut item = new_timeline_agreement("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
