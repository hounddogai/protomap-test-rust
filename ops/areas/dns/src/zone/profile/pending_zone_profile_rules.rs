use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending zone profile must pass. Check returns its problems, or none.
pub trait PendingZoneProfileRule: std::fmt::Debug {
    fn check(&self, item: &PendingZoneProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneProfileAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PendingZoneProfileRule for PendingZoneProfileAllowedLabelRule {
    fn check(&self, item: &PendingZoneProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneProfileRequiredNameRule {
    pub message: String,
}

impl PendingZoneProfileRule for PendingZoneProfileRequiredNameRule {
    fn check(&self, item: &PendingZoneProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingZoneProfileQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingZoneProfileRule for PendingZoneProfileQuotaUnitsRangeRule {
    fn check(&self, item: &PendingZoneProfile) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending zone profile.
#[derive(Debug, Clone)]
pub struct PendingZoneProfileValidator {
    pub rules: Vec<Rc<dyn PendingZoneProfileRule>>,
}

impl PendingZoneProfileValidator {
    pub fn validate(&self, item: &PendingZoneProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingZoneProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingZoneProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_zone_profile_validator() -> PendingZoneProfileValidator {
    PendingZoneProfileValidator {
        rules: vec![
            Rc::new(PendingZoneProfileAllowedLabelRule {
                allowed: vec!["batched".to_string(), "seasonal".to_string(), "regional".to_string()],
            }) as Rc<dyn PendingZoneProfileRule>,
            Rc::new(PendingZoneProfileRequiredNameRule {
                message: "pending zone profile needs this field".to_string(),
            }),
            Rc::new(PendingZoneProfileQuotaUnitsRangeRule { min: 16, max: 120 }),
        ],
    }
}
