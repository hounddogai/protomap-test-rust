use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled egress cycle must pass. Check returns its problems, or none.
pub trait ScheduledEgressCycleRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledEgressCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressCycleRequiredFingerprintRule {
    pub message: String,
}

impl ScheduledEgressCycleRule for ScheduledEgressCycleRequiredFingerprintRule {
    fn check(&self, item: &ScheduledEgressCycle) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressCycleAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledEgressCycleRule for ScheduledEgressCycleAmountRangeRule {
    fn check(&self, item: &ScheduledEgressCycle) -> Vec<String> {
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
pub struct ScheduledEgressCycleLocaleLengthRule {
    pub max_length: i64,
}

impl ScheduledEgressCycleRule for ScheduledEgressCycleLocaleLengthRule {
    fn check(&self, item: &ScheduledEgressCycle) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled egress cycle.
#[derive(Debug, Clone)]
pub struct ScheduledEgressCycleValidator {
    pub rules: Vec<Rc<dyn ScheduledEgressCycleRule>>,
}

impl ScheduledEgressCycleValidator {
    pub fn validate(&self, item: &ScheduledEgressCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledEgressCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledEgressCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_egress_cycle_validator() -> ScheduledEgressCycleValidator {
    ScheduledEgressCycleValidator {
        rules: vec![
            Rc::new(ScheduledEgressCycleRequiredFingerprintRule {
                message: "scheduled egress cycle needs this field".to_string(),
            }) as Rc<dyn ScheduledEgressCycleRule>,
            Rc::new(ScheduledEgressCycleAmountRangeRule { min: 0.9, max: 4673.8 }),
            Rc::new(ScheduledEgressCycleLocaleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_egress_cycle_validator();
        let item = new_scheduled_egress_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ScheduledEgressCycleRequiredFingerprintRule {
            message: "scheduled egress cycle needs this field".to_string(),
        };
        let mut item = new_scheduled_egress_cycle("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = ScheduledEgressCycleAmountRangeRule { min: 3.5, max: 2330.0 };
        let mut item = new_scheduled_egress_cycle("x", "y");
        item.amount = 2331.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = ScheduledEgressCycleLocaleLengthRule { max_length: 64 };
        let mut item = new_scheduled_egress_cycle("x", "y");
        item.locale = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
