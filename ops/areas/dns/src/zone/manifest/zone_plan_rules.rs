use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone plan must pass. Check returns its problems, or none.
pub trait ZonePlanRule: std::fmt::Debug {
    fn check(&self, item: &ZonePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZonePlanAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl ZonePlanRule for ZonePlanAllowedCountryRule {
    fn check(&self, item: &ZonePlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is too long", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZonePlanNameLengthRule {
    pub max_length: i64,
}

impl ZonePlanRule for ZonePlanNameLengthRule {
    fn check(&self, item: &ZonePlan) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZonePlanCodePrefixRule {
    pub prefix: String,
}

impl ZonePlanRule for ZonePlanCodePrefixRule {
    fn check(&self, item: &ZonePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZonePlanArchivedGuardRule {
    pub limit: i64,
}

impl ZonePlanRule for ZonePlanArchivedGuardRule {
    fn check(&self, item: &ZonePlan) -> Vec<String> {
        if item.archived && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while archived",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZonePlanCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ZonePlanRule for ZonePlanCreatedAtRangeRule {
    fn check(&self, item: &ZonePlan) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone plan.
#[derive(Debug, Clone)]
pub struct ZonePlanValidator {
    pub rules: Vec<Rc<dyn ZonePlanRule>>,
}

impl ZonePlanValidator {
    pub fn validate(&self, item: &ZonePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZonePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZonePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_plan_validator() -> ZonePlanValidator {
    ZonePlanValidator {
        rules: vec![
            Rc::new(ZonePlanAllowedCountryRule {
                allowed: vec!["capped".to_string()],
            }) as Rc<dyn ZonePlanRule>,
            Rc::new(ZonePlanNameLengthRule { max_length: 255 }),
            Rc::new(ZonePlanCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ZonePlanArchivedGuardRule { limit: 64 }),
            Rc::new(ZonePlanCreatedAtRangeRule { min: 32, max: 255 }),
        ],
    }
}
