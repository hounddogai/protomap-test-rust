use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled verifier summary must pass. Check returns its problems, or none.
pub trait ScheduledVerifierSummaryRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledVerifierSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierSummaryRequiredTenantIdRule {
    pub message: String,
}

impl ScheduledVerifierSummaryRule for ScheduledVerifierSummaryRequiredTenantIdRule {
    fn check(&self, item: &ScheduledVerifierSummary) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierSummaryRequiredLabelRule {
    pub message: String,
}

impl ScheduledVerifierSummaryRule for ScheduledVerifierSummaryRequiredLabelRule {
    fn check(&self, item: &ScheduledVerifierSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierSummaryLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledVerifierSummaryRule for ScheduledVerifierSummaryLimitRangeRule {
    fn check(&self, item: &ScheduledVerifierSummary) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierSummaryAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl ScheduledVerifierSummaryRule for ScheduledVerifierSummaryAllowedTimezoneRule {
    fn check(&self, item: &ScheduledVerifierSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is below minimum", item.timezone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled verifier summary.
#[derive(Debug, Clone)]
pub struct ScheduledVerifierSummaryValidator {
    pub rules: Vec<Rc<dyn ScheduledVerifierSummaryRule>>,
}

impl ScheduledVerifierSummaryValidator {
    pub fn validate(&self, item: &ScheduledVerifierSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledVerifierSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledVerifierSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_verifier_summary_validator() -> ScheduledVerifierSummaryValidator {
    ScheduledVerifierSummaryValidator {
        rules: vec![
            Rc::new(ScheduledVerifierSummaryRequiredTenantIdRule {
                message: "scheduled verifier summary needs an entry".to_string(),
            }) as Rc<dyn ScheduledVerifierSummaryRule>,
            Rc::new(ScheduledVerifierSummaryRequiredLabelRule {
                message: "scheduled verifier summary needs a value".to_string(),
            }),
            Rc::new(ScheduledVerifierSummaryLimitRangeRule { min: 32, max: 3600 }),
            Rc::new(ScheduledVerifierSummaryAllowedTimezoneRule {
                allowed: vec!["rolling".to_string()],
            }),
        ],
    }
}
