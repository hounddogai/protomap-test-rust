use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone score must pass. Check returns its problems, or none.
pub trait ZoneScoreRule: std::fmt::Debug {
    fn check(&self, item: &ZoneScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneScoreCodeLengthRule {
    pub max_length: i64,
}

impl ZoneScoreRule for ZoneScoreCodeLengthRule {
    fn check(&self, item: &ZoneScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneScoreAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ZoneScoreRule for ZoneScoreAllowedCodeRule {
    fn check(&self, item: &ZoneScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneScoreCountryLengthRule {
    pub max_length: i64,
}

impl ZoneScoreRule for ZoneScoreCountryLengthRule {
    fn check(&self, item: &ZoneScore) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneScoreRequiredNameRule {
    pub message: String,
}

impl ZoneScoreRule for ZoneScoreRequiredNameRule {
    fn check(&self, item: &ZoneScore) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneScoreRequiredCodeRule {
    pub message: String,
}

impl ZoneScoreRule for ZoneScoreRequiredCodeRule {
    fn check(&self, item: &ZoneScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneScoreNameLengthRule {
    pub max_length: i64,
}

impl ZoneScoreRule for ZoneScoreNameLengthRule {
    fn check(&self, item: &ZoneScore) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone score.
#[derive(Debug, Clone)]
pub struct ZoneScoreValidator {
    pub rules: Vec<Rc<dyn ZoneScoreRule>>,
}

impl ZoneScoreValidator {
    pub fn validate(&self, item: &ZoneScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_score_validator() -> ZoneScoreValidator {
    ZoneScoreValidator {
        rules: vec![
            Rc::new(ZoneScoreCodeLengthRule { max_length: 255 }) as Rc<dyn ZoneScoreRule>,
            Rc::new(ZoneScoreAllowedCodeRule {
                allowed: vec!["batched".to_string(), "default".to_string()],
            }),
            Rc::new(ZoneScoreCountryLengthRule { max_length: 3600 }),
            Rc::new(ZoneScoreRequiredNameRule {
                message: "zone score needs this field".to_string(),
            }),
            Rc::new(ZoneScoreRequiredCodeRule {
                message: "zone score needs a value".to_string(),
            }),
            Rc::new(ZoneScoreNameLengthRule { max_length: 86400 }),
        ],
    }
}
