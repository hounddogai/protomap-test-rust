use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling zone manifest must pass. Check returns its problems, or none.
pub trait RollingZoneManifestRule: std::fmt::Debug {
    fn check(&self, item: &RollingZoneManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneManifestAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl RollingZoneManifestRule for RollingZoneManifestAllowedCurrencyRule {
    fn check(&self, item: &RollingZoneManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is stale", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneManifestTitleLengthRule {
    pub max_length: i64,
}

impl RollingZoneManifestRule for RollingZoneManifestTitleLengthRule {
    fn check(&self, item: &RollingZoneManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneManifestCodeLengthRule {
    pub max_length: i64,
}

impl RollingZoneManifestRule for RollingZoneManifestCodeLengthRule {
    fn check(&self, item: &RollingZoneManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneManifestRequiredCurrencyRule {
    pub message: String,
}

impl RollingZoneManifestRule for RollingZoneManifestRequiredCurrencyRule {
    fn check(&self, item: &RollingZoneManifest) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneManifestRequiredCodeRule {
    pub message: String,
}

impl RollingZoneManifestRule for RollingZoneManifestRequiredCodeRule {
    fn check(&self, item: &RollingZoneManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling zone manifest.
#[derive(Debug, Clone)]
pub struct RollingZoneManifestValidator {
    pub rules: Vec<Rc<dyn RollingZoneManifestRule>>,
}

impl RollingZoneManifestValidator {
    pub fn validate(&self, item: &RollingZoneManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingZoneManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingZoneManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_zone_manifest_validator() -> RollingZoneManifestValidator {
    RollingZoneManifestValidator {
        rules: vec![
            Rc::new(RollingZoneManifestAllowedCurrencyRule {
                allowed: vec!["rolling".to_string()],
            }) as Rc<dyn RollingZoneManifestRule>,
            Rc::new(RollingZoneManifestTitleLengthRule { max_length: 64 }),
            Rc::new(RollingZoneManifestCodeLengthRule { max_length: 80 }),
            Rc::new(RollingZoneManifestRequiredCurrencyRule {
                message: "rolling zone manifest needs an entry".to_string(),
            }),
            Rc::new(RollingZoneManifestRequiredCodeRule {
                message: "rolling zone manifest needs an entry".to_string(),
            }),
        ],
    }
}
