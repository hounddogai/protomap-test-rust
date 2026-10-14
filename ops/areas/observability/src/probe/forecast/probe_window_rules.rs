use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a probe window must pass. Check returns its problems, or none.
pub trait ProbeWindowRule: std::fmt::Debug {
    fn check(&self, item: &ProbeWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeWindowRequiredTitleRule {
    pub message: String,
}

impl ProbeWindowRule for ProbeWindowRequiredTitleRule {
    fn check(&self, item: &ProbeWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeWindowAudienceLengthRule {
    pub max_length: i64,
}

impl ProbeWindowRule for ProbeWindowAudienceLengthRule {
    fn check(&self, item: &ProbeWindow) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeWindowGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeWindowRule for ProbeWindowGraceDaysRangeRule {
    fn check(&self, item: &ProbeWindow) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeWindowAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ProbeWindowRule for ProbeWindowAllowedCodeRule {
    fn check(&self, item: &ProbeWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeWindowCodePrefixRule {
    pub prefix: String,
}

impl ProbeWindowRule for ProbeWindowCodePrefixRule {
    fn check(&self, item: &ProbeWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeWindowIssuerLengthRule {
    pub max_length: i64,
}

impl ProbeWindowRule for ProbeWindowIssuerLengthRule {
    fn check(&self, item: &ProbeWindow) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe window.
#[derive(Debug, Clone)]
pub struct ProbeWindowValidator {
    pub rules: Vec<Rc<dyn ProbeWindowRule>>,
}

impl ProbeWindowValidator {
    pub fn validate(&self, item: &ProbeWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_window_validator() -> ProbeWindowValidator {
    ProbeWindowValidator {
        rules: vec![
            Rc::new(ProbeWindowRequiredTitleRule {
                message: "probe window needs a value".to_string(),
            }) as Rc<dyn ProbeWindowRule>,
            Rc::new(ProbeWindowAudienceLengthRule { max_length: 1000 }),
            Rc::new(ProbeWindowGraceDaysRangeRule { min: 120, max: 1000 }),
            Rc::new(ProbeWindowAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "lenient".to_string(), "regional".to_string()],
            }),
            Rc::new(ProbeWindowCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ProbeWindowIssuerLengthRule { max_length: 32 }),
        ],
    }
}
