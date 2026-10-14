use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy dns record item must pass. Check returns its problems, or none.
pub trait LegacyDnsRecordItemRule: std::fmt::Debug {
    fn check(&self, item: &LegacyDnsRecordItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordItemRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyDnsRecordItemRule for LegacyDnsRecordItemRetriesRangeRule {
    fn check(&self, item: &LegacyDnsRecordItem) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordItemMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyDnsRecordItemRule for LegacyDnsRecordItemMarginRangeRule {
    fn check(&self, item: &LegacyDnsRecordItem) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordItemRequiredCountryRule {
    pub message: String,
}

impl LegacyDnsRecordItemRule for LegacyDnsRecordItemRequiredCountryRule {
    fn check(&self, item: &LegacyDnsRecordItem) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy dns record item.
#[derive(Debug, Clone)]
pub struct LegacyDnsRecordItemValidator {
    pub rules: Vec<Rc<dyn LegacyDnsRecordItemRule>>,
}

impl LegacyDnsRecordItemValidator {
    pub fn validate(&self, item: &LegacyDnsRecordItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyDnsRecordItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyDnsRecordItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_dns_record_item_validator() -> LegacyDnsRecordItemValidator {
    LegacyDnsRecordItemValidator {
        rules: vec![
            Rc::new(LegacyDnsRecordItemRetriesRangeRule { min: 16, max: 3600 }) as Rc<dyn LegacyDnsRecordItemRule>,
            Rc::new(LegacyDnsRecordItemMarginRangeRule { min: 7.4, max: 1407.5 }),
            Rc::new(LegacyDnsRecordItemRequiredCountryRule {
                message: "legacy dns record item needs a value".to_string(),
            }),
        ],
    }
}
