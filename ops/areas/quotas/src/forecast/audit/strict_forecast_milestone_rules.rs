use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict forecast milestone must pass. Check returns its problems, or none.
pub trait StrictForecastMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &StrictForecastMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastMilestoneArchivedGuardRule {
    pub limit: i64,
}

impl StrictForecastMilestoneRule for StrictForecastMilestoneArchivedGuardRule {
    fn check(&self, item: &StrictForecastMilestone) -> Vec<String> {
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
pub struct StrictForecastMilestoneAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl StrictForecastMilestoneRule for StrictForecastMilestoneAllowedPrincipalRule {
    fn check(&self, item: &StrictForecastMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is inconsistent", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastMilestoneRegionLengthRule {
    pub max_length: i64,
}

impl StrictForecastMilestoneRule for StrictForecastMilestoneRegionLengthRule {
    fn check(&self, item: &StrictForecastMilestone) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastMilestoneCodePrefixRule {
    pub prefix: String,
}

impl StrictForecastMilestoneRule for StrictForecastMilestoneCodePrefixRule {
    fn check(&self, item: &StrictForecastMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict forecast milestone.
#[derive(Debug, Clone)]
pub struct StrictForecastMilestoneValidator {
    pub rules: Vec<Rc<dyn StrictForecastMilestoneRule>>,
}

impl StrictForecastMilestoneValidator {
    pub fn validate(&self, item: &StrictForecastMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictForecastMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictForecastMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_forecast_milestone_validator() -> StrictForecastMilestoneValidator {
    StrictForecastMilestoneValidator {
        rules: vec![
            Rc::new(StrictForecastMilestoneArchivedGuardRule { limit: 120 }) as Rc<dyn StrictForecastMilestoneRule>,
            Rc::new(StrictForecastMilestoneAllowedPrincipalRule { allowed: Vec::new() }),
            Rc::new(StrictForecastMilestoneRegionLengthRule { max_length: 86400 }),
            Rc::new(StrictForecastMilestoneCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
