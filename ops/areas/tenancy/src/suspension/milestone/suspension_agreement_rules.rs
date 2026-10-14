use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension agreement must pass. Check returns its problems, or none.
pub trait SuspensionAgreementRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAgreementWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionAgreementRule for SuspensionAgreementWeightRangeRule {
    fn check(&self, item: &SuspensionAgreement) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAgreementCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionAgreementRule for SuspensionAgreementCapacityRangeRule {
    fn check(&self, item: &SuspensionAgreement) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAgreementRequiredNameRule {
    pub message: String,
}

impl SuspensionAgreementRule for SuspensionAgreementRequiredNameRule {
    fn check(&self, item: &SuspensionAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAgreementCodeLengthRule {
    pub max_length: i64,
}

impl SuspensionAgreementRule for SuspensionAgreementCodeLengthRule {
    fn check(&self, item: &SuspensionAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAgreementNameLengthRule {
    pub max_length: i64,
}

impl SuspensionAgreementRule for SuspensionAgreementNameLengthRule {
    fn check(&self, item: &SuspensionAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension agreement.
#[derive(Debug, Clone)]
pub struct SuspensionAgreementValidator {
    pub rules: Vec<Rc<dyn SuspensionAgreementRule>>,
}

impl SuspensionAgreementValidator {
    pub fn validate(&self, item: &SuspensionAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_agreement_validator() -> SuspensionAgreementValidator {
    SuspensionAgreementValidator {
        rules: vec![
            Rc::new(SuspensionAgreementWeightRangeRule { min: 6.8, max: 2938.9 }) as Rc<dyn SuspensionAgreementRule>,
            Rc::new(SuspensionAgreementCapacityRangeRule { min: 255, max: 3600 }),
            Rc::new(SuspensionAgreementRequiredNameRule {
                message: "suspension agreement needs an entry".to_string(),
            }),
            Rc::new(SuspensionAgreementCodeLengthRule { max_length: 120 }),
            Rc::new(SuspensionAgreementNameLengthRule { max_length: 3600 }),
        ],
    }
}
