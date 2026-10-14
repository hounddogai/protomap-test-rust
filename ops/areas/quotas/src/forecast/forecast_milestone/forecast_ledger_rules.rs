use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast ledger must pass. Check returns its problems, or none.
pub trait ForecastLedgerRule: std::fmt::Debug {
    fn check(&self, item: &ForecastLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastLedgerPhoneLengthRule {
    pub max_length: i64,
}

impl ForecastLedgerRule for ForecastLedgerPhoneLengthRule {
    fn check(&self, item: &ForecastLedger) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastLedgerTimezoneLengthRule {
    pub max_length: i64,
}

impl ForecastLedgerRule for ForecastLedgerTimezoneLengthRule {
    fn check(&self, item: &ForecastLedger) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastLedgerLabelLengthRule {
    pub max_length: i64,
}

impl ForecastLedgerRule for ForecastLedgerLabelLengthRule {
    fn check(&self, item: &ForecastLedger) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastLedgerAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl ForecastLedgerRule for ForecastLedgerAllowedTimezoneRule {
    fn check(&self, item: &ForecastLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is missing", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastLedgerCodeLengthRule {
    pub max_length: i64,
}

impl ForecastLedgerRule for ForecastLedgerCodeLengthRule {
    fn check(&self, item: &ForecastLedger) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastLedgerCodePrefixRule {
    pub prefix: String,
}

impl ForecastLedgerRule for ForecastLedgerCodePrefixRule {
    fn check(&self, item: &ForecastLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast ledger.
#[derive(Debug, Clone)]
pub struct ForecastLedgerValidator {
    pub rules: Vec<Rc<dyn ForecastLedgerRule>>,
}

impl ForecastLedgerValidator {
    pub fn validate(&self, item: &ForecastLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_ledger_validator() -> ForecastLedgerValidator {
    ForecastLedgerValidator {
        rules: vec![
            Rc::new(ForecastLedgerPhoneLengthRule { max_length: 64 }) as Rc<dyn ForecastLedgerRule>,
            Rc::new(ForecastLedgerTimezoneLengthRule { max_length: 86400 }),
            Rc::new(ForecastLedgerLabelLengthRule { max_length: 3600 }),
            Rc::new(ForecastLedgerAllowedTimezoneRule {
                allowed: vec!["capped".to_string()],
            }),
            Rc::new(ForecastLedgerCodeLengthRule { max_length: 80 }),
            Rc::new(ForecastLedgerCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}
