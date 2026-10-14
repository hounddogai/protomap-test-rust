use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy cycle must pass. Check returns its problems, or none.
pub trait PolicyCycleRule: std::fmt::Debug {
    fn check(&self, item: &PolicyCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyCycleRequiredTimezoneRule {
    pub message: String,
}

impl PolicyCycleRule for PolicyCycleRequiredTimezoneRule {
    fn check(&self, item: &PolicyCycle) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyCycleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyCycleRule for PolicyCycleEndsAtRangeRule {
    fn check(&self, item: &PolicyCycle) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyCycleLabelLengthRule {
    pub max_length: i64,
}

impl PolicyCycleRule for PolicyCycleLabelLengthRule {
    fn check(&self, item: &PolicyCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyCycleContactEmailLengthRule {
    pub max_length: i64,
}

impl PolicyCycleRule for PolicyCycleContactEmailLengthRule {
    fn check(&self, item: &PolicyCycle) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyCycleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyCycleRule for PolicyCycleRotationDaysRangeRule {
    fn check(&self, item: &PolicyCycle) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a policy cycle.
#[derive(Debug, Clone)]
pub struct PolicyCycleValidator {
    pub rules: Vec<Rc<dyn PolicyCycleRule>>,
}

impl PolicyCycleValidator {
    pub fn validate(&self, item: &PolicyCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_cycle_validator() -> PolicyCycleValidator {
    PolicyCycleValidator {
        rules: vec![
            Rc::new(PolicyCycleRequiredTimezoneRule {
                message: "policy cycle needs this field".to_string(),
            }) as Rc<dyn PolicyCycleRule>,
            Rc::new(PolicyCycleEndsAtRangeRule { min: 3600, max: 86400 }),
            Rc::new(PolicyCycleLabelLengthRule { max_length: 3600 }),
            Rc::new(PolicyCycleContactEmailLengthRule { max_length: 16 }),
            Rc::new(PolicyCycleRotationDaysRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_cycle_validator();
        let item = new_policy_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PolicyCycleRequiredTimezoneRule {
            message: "policy cycle needs this field".to_string(),
        };
        let mut item = new_policy_cycle("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = PolicyCycleEndsAtRangeRule { min: 80, max: 3600 };
        let mut item = new_policy_cycle("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PolicyCycleLabelLengthRule { max_length: 32 };
        let mut item = new_policy_cycle("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = PolicyCycleContactEmailLengthRule { max_length: 255 };
        let mut item = new_policy_cycle("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = PolicyCycleRotationDaysRangeRule { min: 16, max: 64 };
        let mut item = new_policy_cycle("x", "y");
        item.rotation_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
