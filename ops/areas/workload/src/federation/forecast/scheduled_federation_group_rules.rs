use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled federation group must pass. Check returns its problems, or none.
pub trait ScheduledFederationGroupRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFederationGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationGroupAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl ScheduledFederationGroupRule for ScheduledFederationGroupAllowedCategoryRule {
    fn check(&self, item: &ScheduledFederationGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is expired", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationGroupPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledFederationGroupRule for ScheduledFederationGroupPortRangeRule {
    fn check(&self, item: &ScheduledFederationGroup) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationGroupRequiredDescriptionRule {
    pub message: String,
}

impl ScheduledFederationGroupRule for ScheduledFederationGroupRequiredDescriptionRule {
    fn check(&self, item: &ScheduledFederationGroup) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationGroupCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledFederationGroupRule for ScheduledFederationGroupCodeLengthRule {
    fn check(&self, item: &ScheduledFederationGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationGroupCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledFederationGroupRule for ScheduledFederationGroupCreatedAtRangeRule {
    fn check(&self, item: &ScheduledFederationGroup) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationGroupChecksumLengthRule {
    pub max_length: i64,
}

impl ScheduledFederationGroupRule for ScheduledFederationGroupChecksumLengthRule {
    fn check(&self, item: &ScheduledFederationGroup) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled federation group.
#[derive(Debug, Clone)]
pub struct ScheduledFederationGroupValidator {
    pub rules: Vec<Rc<dyn ScheduledFederationGroupRule>>,
}

impl ScheduledFederationGroupValidator {
    pub fn validate(&self, item: &ScheduledFederationGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledFederationGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFederationGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_federation_group_validator() -> ScheduledFederationGroupValidator {
    ScheduledFederationGroupValidator {
        rules: vec![
            Rc::new(ScheduledFederationGroupAllowedCategoryRule {
                allowed: vec!["fallback".to_string()],
            }) as Rc<dyn ScheduledFederationGroupRule>,
            Rc::new(ScheduledFederationGroupPortRangeRule { min: 64, max: 3600 }),
            Rc::new(ScheduledFederationGroupRequiredDescriptionRule {
                message: "scheduled federation group needs a value".to_string(),
            }),
            Rc::new(ScheduledFederationGroupCodeLengthRule { max_length: 255 }),
            Rc::new(ScheduledFederationGroupCreatedAtRangeRule { min: 120, max: 86400 }),
            Rc::new(ScheduledFederationGroupChecksumLengthRule { max_length: 3600 }),
        ],
    }
}
