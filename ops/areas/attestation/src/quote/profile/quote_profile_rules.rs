use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote profile must pass. Check returns its problems, or none.
pub trait QuoteProfileRule: std::fmt::Debug {
    fn check(&self, item: &QuoteProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProfileVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteProfileRule for QuoteProfileVersionRangeRule {
    fn check(&self, item: &QuoteProfile) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProfileNameLengthRule {
    pub max_length: i64,
}

impl QuoteProfileRule for QuoteProfileNameLengthRule {
    fn check(&self, item: &QuoteProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProfileRequiredCountryRule {
    pub message: String,
}

impl QuoteProfileRule for QuoteProfileRequiredCountryRule {
    fn check(&self, item: &QuoteProfile) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote profile.
#[derive(Debug, Clone)]
pub struct QuoteProfileValidator {
    pub rules: Vec<Rc<dyn QuoteProfileRule>>,
}

impl QuoteProfileValidator {
    pub fn validate(&self, item: &QuoteProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_profile_validator() -> QuoteProfileValidator {
    QuoteProfileValidator {
        rules: vec![
            Rc::new(QuoteProfileVersionRangeRule { min: 16, max: 64 }) as Rc<dyn QuoteProfileRule>,
            Rc::new(QuoteProfileNameLengthRule { max_length: 255 }),
            Rc::new(QuoteProfileRequiredCountryRule {
                message: "quote profile needs this field".to_string(),
            }),
        ],
    }
}
