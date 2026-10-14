use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional zone override must pass. Check returns its problems, or none.
pub trait RegionalZoneOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RegionalZoneOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneOverrideRequiredCodeRule {
    pub message: String,
}

impl RegionalZoneOverrideRule for RegionalZoneOverrideRequiredCodeRule {
    fn check(&self, item: &RegionalZoneOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneOverrideFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalZoneOverrideRule for RegionalZoneOverrideFloorValueRangeRule {
    fn check(&self, item: &RegionalZoneOverride) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneOverrideDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalZoneOverrideRule for RegionalZoneOverrideDurationMinutesRangeRule {
    fn check(&self, item: &RegionalZoneOverride) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneOverrideCodeLengthRule {
    pub max_length: i64,
}

impl RegionalZoneOverrideRule for RegionalZoneOverrideCodeLengthRule {
    fn check(&self, item: &RegionalZoneOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneOverrideHostLengthRule {
    pub max_length: i64,
}

impl RegionalZoneOverrideRule for RegionalZoneOverrideHostLengthRule {
    fn check(&self, item: &RegionalZoneOverride) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneOverrideUnitLengthRule {
    pub max_length: i64,
}

impl RegionalZoneOverrideRule for RegionalZoneOverrideUnitLengthRule {
    fn check(&self, item: &RegionalZoneOverride) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional zone override.
#[derive(Debug, Clone)]
pub struct RegionalZoneOverrideValidator {
    pub rules: Vec<Rc<dyn RegionalZoneOverrideRule>>,
}

impl RegionalZoneOverrideValidator {
    pub fn validate(&self, item: &RegionalZoneOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalZoneOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalZoneOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_zone_override_validator() -> RegionalZoneOverrideValidator {
    RegionalZoneOverrideValidator {
        rules: vec![
            Rc::new(RegionalZoneOverrideRequiredCodeRule {
                message: "regional zone override needs this field".to_string(),
            }) as Rc<dyn RegionalZoneOverrideRule>,
            Rc::new(RegionalZoneOverrideFloorValueRangeRule { min: 4.8, max: 633.3 }),
            Rc::new(RegionalZoneOverrideDurationMinutesRangeRule { min: 64, max: 3600 }),
            Rc::new(RegionalZoneOverrideCodeLengthRule { max_length: 1000 }),
            Rc::new(RegionalZoneOverrideHostLengthRule { max_length: 3600 }),
            Rc::new(RegionalZoneOverrideUnitLengthRule { max_length: 86400 }),
        ],
    }
}
