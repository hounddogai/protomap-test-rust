use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain agreement must pass. Check returns its problems, or none.
pub trait TrustDomainAgreementRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAgreementCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainAgreementRule for TrustDomainAgreementCodeLengthRule {
    fn check(&self, item: &TrustDomainAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAgreementBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainAgreementRule for TrustDomainAgreementBurstRangeRule {
    fn check(&self, item: &TrustDomainAgreement) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAgreementNameLengthRule {
    pub max_length: i64,
}

impl TrustDomainAgreementRule for TrustDomainAgreementNameLengthRule {
    fn check(&self, item: &TrustDomainAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAgreementAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainAgreementRule for TrustDomainAgreementAmountRangeRule {
    fn check(&self, item: &TrustDomainAgreement) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAgreementGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainAgreementRule for TrustDomainAgreementGraceDaysRangeRule {
    fn check(&self, item: &TrustDomainAgreement) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain agreement.
#[derive(Debug, Clone)]
pub struct TrustDomainAgreementValidator {
    pub rules: Vec<Rc<dyn TrustDomainAgreementRule>>,
}

impl TrustDomainAgreementValidator {
    pub fn validate(&self, item: &TrustDomainAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_agreement_validator() -> TrustDomainAgreementValidator {
    TrustDomainAgreementValidator {
        rules: vec![
            Rc::new(TrustDomainAgreementCodeLengthRule { max_length: 86400 }) as Rc<dyn TrustDomainAgreementRule>,
            Rc::new(TrustDomainAgreementBurstRangeRule { min: 120, max: 255 }),
            Rc::new(TrustDomainAgreementNameLengthRule { max_length: 64 }),
            Rc::new(TrustDomainAgreementAmountRangeRule { min: 7.2, max: 2531.6 }),
            Rc::new(TrustDomainAgreementGraceDaysRangeRule { min: 80, max: 120 }),
        ],
    }
}
