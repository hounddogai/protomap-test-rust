use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling segment agreement must pass. Check returns its problems, or none.
pub trait RollingSegmentAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RollingSegmentAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentAgreementCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingSegmentAgreementRule for RollingSegmentAgreementCeilingRangeRule {
    fn check(&self, item: &RollingSegmentAgreement) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentAgreementRequiredCodeRule {
    pub message: String,
}

impl RollingSegmentAgreementRule for RollingSegmentAgreementRequiredCodeRule {
    fn check(&self, item: &RollingSegmentAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentAgreementCodePrefixRule {
    pub prefix: String,
}

impl RollingSegmentAgreementRule for RollingSegmentAgreementCodePrefixRule {
    fn check(&self, item: &RollingSegmentAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling segment agreement.
#[derive(Debug, Clone)]
pub struct RollingSegmentAgreementValidator {
    pub rules: Vec<Rc<dyn RollingSegmentAgreementRule>>,
}

impl RollingSegmentAgreementValidator {
    pub fn validate(&self, item: &RollingSegmentAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingSegmentAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSegmentAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_segment_agreement_validator() -> RollingSegmentAgreementValidator {
    RollingSegmentAgreementValidator {
        rules: vec![
            Rc::new(RollingSegmentAgreementCeilingRangeRule { min: 8.0, max: 510.2 })
                as Rc<dyn RollingSegmentAgreementRule>,
            Rc::new(RollingSegmentAgreementRequiredCodeRule {
                message: "rolling segment agreement needs a value".to_string(),
            }),
            Rc::new(RollingSegmentAgreementCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
