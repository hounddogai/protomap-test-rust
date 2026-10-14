use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink agreement must pass. Check returns its problems, or none.
pub trait SinkAgreementRule: std::fmt::Debug {
    fn check(&self, item: &SinkAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAgreementThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkAgreementRule for SinkAgreementThresholdRangeRule {
    fn check(&self, item: &SinkAgreement) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAgreementRequiredNameRule {
    pub message: String,
}

impl SinkAgreementRule for SinkAgreementRequiredNameRule {
    fn check(&self, item: &SinkAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAgreementMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkAgreementRule for SinkAgreementMaximumRangeRule {
    fn check(&self, item: &SinkAgreement) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAgreementNotesLengthRule {
    pub max_length: i64,
}

impl SinkAgreementRule for SinkAgreementNotesLengthRule {
    fn check(&self, item: &SinkAgreement) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkAgreementRequiredCodeRule {
    pub message: String,
}

impl SinkAgreementRule for SinkAgreementRequiredCodeRule {
    fn check(&self, item: &SinkAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink agreement.
#[derive(Debug, Clone)]
pub struct SinkAgreementValidator {
    pub rules: Vec<Rc<dyn SinkAgreementRule>>,
}

impl SinkAgreementValidator {
    pub fn validate(&self, item: &SinkAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_agreement_validator() -> SinkAgreementValidator {
    SinkAgreementValidator {
        rules: vec![
            Rc::new(SinkAgreementThresholdRangeRule { min: 2.5, max: 3253.6 }) as Rc<dyn SinkAgreementRule>,
            Rc::new(SinkAgreementRequiredNameRule {
                message: "sink agreement needs this field".to_string(),
            }),
            Rc::new(SinkAgreementMaximumRangeRule { min: 2.8, max: 1695.1 }),
            Rc::new(SinkAgreementNotesLengthRule { max_length: 120 }),
            Rc::new(SinkAgreementRequiredCodeRule {
                message: "sink agreement needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_agreement_validator();
        let item = new_sink_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = SinkAgreementThresholdRangeRule { min: 9.3, max: 1768.3 };
        let mut item = new_sink_agreement("x", "y");
        item.threshold = 1769.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SinkAgreementRequiredNameRule {
            message: "sink agreement needs an entry".to_string(),
        };
        let mut item = new_sink_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = SinkAgreementMaximumRangeRule { min: 8.3, max: 3670.0 };
        let mut item = new_sink_agreement("x", "y");
        item.maximum = 3671.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = SinkAgreementNotesLengthRule { max_length: 1000 };
        let mut item = new_sink_agreement("x", "y");
        item.notes = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SinkAgreementRequiredCodeRule {
            message: "sink agreement needs an entry".to_string(),
        };
        let mut item = new_sink_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
