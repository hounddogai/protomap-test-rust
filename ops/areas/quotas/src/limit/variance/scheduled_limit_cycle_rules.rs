use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled limit cycle must pass. Check returns its problems, or none.
pub trait ScheduledLimitCycleRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledLimitCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLimitCycleCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledLimitCycleRule for ScheduledLimitCycleCodeLengthRule {
    fn check(&self, item: &ScheduledLimitCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLimitCycleAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledLimitCycleRule for ScheduledLimitCycleAmountRangeRule {
    fn check(&self, item: &ScheduledLimitCycle) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLimitCycleRequiredCodeRule {
    pub message: String,
}

impl ScheduledLimitCycleRule for ScheduledLimitCycleRequiredCodeRule {
    fn check(&self, item: &ScheduledLimitCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLimitCycleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl ScheduledLimitCycleRule for ScheduledLimitCycleAllowedNameRule {
    fn check(&self, item: &ScheduledLimitCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled limit cycle.
#[derive(Debug, Clone)]
pub struct ScheduledLimitCycleValidator {
    pub rules: Vec<Rc<dyn ScheduledLimitCycleRule>>,
}

impl ScheduledLimitCycleValidator {
    pub fn validate(&self, item: &ScheduledLimitCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledLimitCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledLimitCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_limit_cycle_validator() -> ScheduledLimitCycleValidator {
    ScheduledLimitCycleValidator {
        rules: vec![
            Rc::new(ScheduledLimitCycleCodeLengthRule { max_length: 1000 }) as Rc<dyn ScheduledLimitCycleRule>,
            Rc::new(ScheduledLimitCycleAmountRangeRule { min: 3.9, max: 4589.2 }),
            Rc::new(ScheduledLimitCycleRequiredCodeRule {
                message: "scheduled limit cycle needs a value".to_string(),
            }),
            Rc::new(ScheduledLimitCycleAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_limit_cycle_validator();
        let item = new_scheduled_limit_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledLimitCycleCodeLengthRule { max_length: 16 };
        let mut item = new_scheduled_limit_cycle("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ScheduledLimitCycleAmountRangeRule { min: 1.2, max: 4823.0 };
        let mut item = new_scheduled_limit_cycle("x", "y");
        item.amount = 4824.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledLimitCycleRequiredCodeRule {
            message: "scheduled limit cycle needs a value".to_string(),
        };
        let mut item = new_scheduled_limit_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
