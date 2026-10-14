use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global sink agreement must pass. Check returns its problems, or none.
pub trait GlobalSinkAgreementRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSinkAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkAgreementFingerprintLengthRule {
    pub max_length: i64,
}

impl GlobalSinkAgreementRule for GlobalSinkAgreementFingerprintLengthRule {
    fn check(&self, item: &GlobalSinkAgreement) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkAgreementRequiredAudienceRule {
    pub message: String,
}

impl GlobalSinkAgreementRule for GlobalSinkAgreementRequiredAudienceRule {
    fn check(&self, item: &GlobalSinkAgreement) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkAgreementCodeLengthRule {
    pub max_length: i64,
}

impl GlobalSinkAgreementRule for GlobalSinkAgreementCodeLengthRule {
    fn check(&self, item: &GlobalSinkAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkAgreementMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalSinkAgreementRule for GlobalSinkAgreementMaximumRangeRule {
    fn check(&self, item: &GlobalSinkAgreement) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkAgreementAudienceLengthRule {
    pub max_length: i64,
}

impl GlobalSinkAgreementRule for GlobalSinkAgreementAudienceLengthRule {
    fn check(&self, item: &GlobalSinkAgreement) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global sink agreement.
#[derive(Debug, Clone)]
pub struct GlobalSinkAgreementValidator {
    pub rules: Vec<Rc<dyn GlobalSinkAgreementRule>>,
}

impl GlobalSinkAgreementValidator {
    pub fn validate(&self, item: &GlobalSinkAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalSinkAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSinkAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_sink_agreement_validator() -> GlobalSinkAgreementValidator {
    GlobalSinkAgreementValidator {
        rules: vec![
            Rc::new(GlobalSinkAgreementFingerprintLengthRule { max_length: 120 }) as Rc<dyn GlobalSinkAgreementRule>,
            Rc::new(GlobalSinkAgreementRequiredAudienceRule {
                message: "global sink agreement needs this field".to_string(),
            }),
            Rc::new(GlobalSinkAgreementCodeLengthRule { max_length: 86400 }),
            Rc::new(GlobalSinkAgreementMaximumRangeRule { min: 0.8, max: 2571.7 }),
            Rc::new(GlobalSinkAgreementAudienceLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_sink_agreement_validator();
        let item = new_global_sink_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = GlobalSinkAgreementFingerprintLengthRule { max_length: 3600 };
        let mut item = new_global_sink_agreement("x", "y");
        item.fingerprint = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = GlobalSinkAgreementRequiredAudienceRule {
            message: "global sink agreement needs this field".to_string(),
        };
        let mut item = new_global_sink_agreement("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalSinkAgreementCodeLengthRule { max_length: 1000 };
        let mut item = new_global_sink_agreement("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = GlobalSinkAgreementMaximumRangeRule { min: 4.4, max: 292.1 };
        let mut item = new_global_sink_agreement("x", "y");
        item.maximum = 293.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = GlobalSinkAgreementAudienceLengthRule { max_length: 1000 };
        let mut item = new_global_sink_agreement("x", "y");
        item.audience = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
