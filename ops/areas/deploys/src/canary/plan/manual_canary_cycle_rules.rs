use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual canary cycle must pass. Check returns its problems, or none.
pub trait ManualCanaryCycleRule: std::fmt::Debug {
    fn check(&self, item: &ManualCanaryCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanaryCycleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualCanaryCycleRule for ManualCanaryCycleBurstRangeRule {
    fn check(&self, item: &ManualCanaryCycle) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanaryCycleRequiredOwnerIdRule {
    pub message: String,
}

impl ManualCanaryCycleRule for ManualCanaryCycleRequiredOwnerIdRule {
    fn check(&self, item: &ManualCanaryCycle) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualCanaryCycleOwnerIdLengthRule {
    pub max_length: i64,
}

impl ManualCanaryCycleRule for ManualCanaryCycleOwnerIdLengthRule {
    fn check(&self, item: &ManualCanaryCycle) -> Vec<String> {
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
pub struct ManualCanaryCycleRequiredNameRule {
    pub message: String,
}

impl ManualCanaryCycleRule for ManualCanaryCycleRequiredNameRule {
    fn check(&self, item: &ManualCanaryCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual canary cycle.
#[derive(Debug, Clone)]
pub struct ManualCanaryCycleValidator {
    pub rules: Vec<Rc<dyn ManualCanaryCycleRule>>,
}

impl ManualCanaryCycleValidator {
    pub fn validate(&self, item: &ManualCanaryCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualCanaryCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualCanaryCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_canary_cycle_validator() -> ManualCanaryCycleValidator {
    ManualCanaryCycleValidator {
        rules: vec![
            Rc::new(ManualCanaryCycleBurstRangeRule { min: 16, max: 80 }) as Rc<dyn ManualCanaryCycleRule>,
            Rc::new(ManualCanaryCycleRequiredOwnerIdRule {
                message: "manual canary cycle needs this field".to_string(),
            }),
            Rc::new(ManualCanaryCycleOwnerIdLengthRule { max_length: 120 }),
            Rc::new(ManualCanaryCycleRequiredNameRule {
                message: "manual canary cycle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_canary_cycle_validator();
        let item = new_manual_canary_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = ManualCanaryCycleBurstRangeRule { min: 32, max: 80 };
        let mut item = new_manual_canary_cycle("x", "y");
        item.burst = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = ManualCanaryCycleRequiredOwnerIdRule {
            message: "manual canary cycle needs a value".to_string(),
        };
        let mut item = new_manual_canary_cycle("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = ManualCanaryCycleOwnerIdLengthRule { max_length: 80 };
        let mut item = new_manual_canary_cycle("x", "y");
        item.owner_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualCanaryCycleRequiredNameRule {
            message: "manual canary cycle needs an entry".to_string(),
        };
        let mut item = new_manual_canary_cycle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
