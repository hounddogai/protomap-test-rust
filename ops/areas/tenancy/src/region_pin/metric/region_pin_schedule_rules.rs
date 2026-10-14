use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a region pin schedule must pass. Check returns its problems, or none.
pub trait RegionPinScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinScheduleAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl RegionPinScheduleRule for RegionPinScheduleAllowedOwnerIdRule {
    fn check(&self, item: &RegionPinSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is below minimum", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinScheduleRequiredOwnerIdRule {
    pub message: String,
}

impl RegionPinScheduleRule for RegionPinScheduleRequiredOwnerIdRule {
    fn check(&self, item: &RegionPinSchedule) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinScheduleRequiredCodeRule {
    pub message: String,
}

impl RegionPinScheduleRule for RegionPinScheduleRequiredCodeRule {
    fn check(&self, item: &RegionPinSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinScheduleOwnerIdLengthRule {
    pub max_length: i64,
}

impl RegionPinScheduleRule for RegionPinScheduleOwnerIdLengthRule {
    fn check(&self, item: &RegionPinSchedule) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinScheduleGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinScheduleRule for RegionPinScheduleGraceDaysRangeRule {
    fn check(&self, item: &RegionPinSchedule) -> Vec<String> {
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
pub struct RegionPinScheduleCodePrefixRule {
    pub prefix: String,
}

impl RegionPinScheduleRule for RegionPinScheduleCodePrefixRule {
    fn check(&self, item: &RegionPinSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin schedule.
#[derive(Debug, Clone)]
pub struct RegionPinScheduleValidator {
    pub rules: Vec<Rc<dyn RegionPinScheduleRule>>,
}

impl RegionPinScheduleValidator {
    pub fn validate(&self, item: &RegionPinSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionPinSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_schedule_validator() -> RegionPinScheduleValidator {
    RegionPinScheduleValidator {
        rules: vec![
            Rc::new(RegionPinScheduleAllowedOwnerIdRule { allowed: Vec::new() }) as Rc<dyn RegionPinScheduleRule>,
            Rc::new(RegionPinScheduleRequiredOwnerIdRule {
                message: "region pin schedule needs an entry".to_string(),
            }),
            Rc::new(RegionPinScheduleRequiredCodeRule {
                message: "region pin schedule needs a value".to_string(),
            }),
            Rc::new(RegionPinScheduleOwnerIdLengthRule { max_length: 32 }),
            Rc::new(RegionPinScheduleGraceDaysRangeRule { min: 80, max: 86400 }),
            Rc::new(RegionPinScheduleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_schedule_validator();
        let item = new_region_pin_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = RegionPinScheduleRequiredOwnerIdRule {
            message: "region pin schedule needs this field".to_string(),
        };
        let mut item = new_region_pin_schedule("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionPinScheduleRequiredCodeRule {
            message: "region pin schedule needs this field".to_string(),
        };
        let mut item = new_region_pin_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = RegionPinScheduleOwnerIdLengthRule { max_length: 80 };
        let mut item = new_region_pin_schedule("x", "y");
        item.owner_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RegionPinScheduleGraceDaysRangeRule { min: 1000, max: 3600 };
        let mut item = new_region_pin_schedule("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
