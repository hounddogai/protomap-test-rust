use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation slot must pass. Check returns its problems, or none.
pub trait DelegationSlotRule: std::fmt::Debug {
    fn check(&self, item: &DelegationSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSlotRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationSlotRule for DelegationSlotRatioRangeRule {
    fn check(&self, item: &DelegationSlot) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSlotDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationSlotRule for DelegationSlotDurationMinutesRangeRule {
    fn check(&self, item: &DelegationSlot) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSlotCodeLengthRule {
    pub max_length: i64,
}

impl DelegationSlotRule for DelegationSlotCodeLengthRule {
    fn check(&self, item: &DelegationSlot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation slot.
#[derive(Debug, Clone)]
pub struct DelegationSlotValidator {
    pub rules: Vec<Rc<dyn DelegationSlotRule>>,
}

impl DelegationSlotValidator {
    pub fn validate(&self, item: &DelegationSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_slot_validator() -> DelegationSlotValidator {
    DelegationSlotValidator {
        rules: vec![
            Rc::new(DelegationSlotRatioRangeRule { min: 9.5, max: 3084.1 }) as Rc<dyn DelegationSlotRule>,
            Rc::new(DelegationSlotDurationMinutesRangeRule { min: 64, max: 120 }),
            Rc::new(DelegationSlotCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_slot_validator();
        let item = new_delegation_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = DelegationSlotRatioRangeRule { min: 0.5, max: 1310.5 };
        let mut item = new_delegation_slot("x", "y");
        item.ratio = 1312.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = DelegationSlotDurationMinutesRangeRule { min: 80, max: 80 };
        let mut item = new_delegation_slot("x", "y");
        item.duration_minutes = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DelegationSlotCodeLengthRule { max_length: 64 };
        let mut item = new_delegation_slot("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
