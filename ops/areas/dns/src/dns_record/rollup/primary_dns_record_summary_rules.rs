use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary dns record summary must pass. Check returns its problems, or none.
pub trait PrimaryDnsRecordSummaryRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryDnsRecordSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordSummaryWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryDnsRecordSummaryRule for PrimaryDnsRecordSummaryWeightRangeRule {
    fn check(&self, item: &PrimaryDnsRecordSummary) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordSummaryDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryDnsRecordSummaryRule for PrimaryDnsRecordSummaryDurationMinutesRangeRule {
    fn check(&self, item: &PrimaryDnsRecordSummary) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordSummaryRequiredTimezoneRule {
    pub message: String,
}

impl PrimaryDnsRecordSummaryRule for PrimaryDnsRecordSummaryRequiredTimezoneRule {
    fn check(&self, item: &PrimaryDnsRecordSummary) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDnsRecordSummaryCodePrefixRule {
    pub prefix: String,
}

impl PrimaryDnsRecordSummaryRule for PrimaryDnsRecordSummaryCodePrefixRule {
    fn check(&self, item: &PrimaryDnsRecordSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary dns record summary.
#[derive(Debug, Clone)]
pub struct PrimaryDnsRecordSummaryValidator {
    pub rules: Vec<Rc<dyn PrimaryDnsRecordSummaryRule>>,
}

impl PrimaryDnsRecordSummaryValidator {
    pub fn validate(&self, item: &PrimaryDnsRecordSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryDnsRecordSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryDnsRecordSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_dns_record_summary_validator() -> PrimaryDnsRecordSummaryValidator {
    PrimaryDnsRecordSummaryValidator {
        rules: vec![
            Rc::new(PrimaryDnsRecordSummaryWeightRangeRule { min: 3.4, max: 4340.8 })
                as Rc<dyn PrimaryDnsRecordSummaryRule>,
            Rc::new(PrimaryDnsRecordSummaryDurationMinutesRangeRule { min: 16, max: 1000 }),
            Rc::new(PrimaryDnsRecordSummaryRequiredTimezoneRule {
                message: "primary dns record summary needs a value".to_string(),
            }),
            Rc::new(PrimaryDnsRecordSummaryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
