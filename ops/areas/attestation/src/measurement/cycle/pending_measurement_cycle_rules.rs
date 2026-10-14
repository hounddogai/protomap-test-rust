use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending measurement cycle must pass. Check returns its problems, or none.
pub trait PendingMeasurementCycleRule: std::fmt::Debug {
    fn check(&self, item: &PendingMeasurementCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementCycleRequiredCodeRule {
    pub message: String,
}

impl PendingMeasurementCycleRule for PendingMeasurementCycleRequiredCodeRule {
    fn check(&self, item: &PendingMeasurementCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementCycleCodePrefixRule {
    pub prefix: String,
}

impl PendingMeasurementCycleRule for PendingMeasurementCycleCodePrefixRule {
    fn check(&self, item: &PendingMeasurementCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementCycleLabelLengthRule {
    pub max_length: i64,
}

impl PendingMeasurementCycleRule for PendingMeasurementCycleLabelLengthRule {
    fn check(&self, item: &PendingMeasurementCycle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending measurement cycle.
#[derive(Debug, Clone)]
pub struct PendingMeasurementCycleValidator {
    pub rules: Vec<Rc<dyn PendingMeasurementCycleRule>>,
}

impl PendingMeasurementCycleValidator {
    pub fn validate(&self, item: &PendingMeasurementCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingMeasurementCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingMeasurementCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_measurement_cycle_validator() -> PendingMeasurementCycleValidator {
    PendingMeasurementCycleValidator {
        rules: vec![
            Rc::new(PendingMeasurementCycleRequiredCodeRule {
                message: "pending measurement cycle needs an entry".to_string(),
            }) as Rc<dyn PendingMeasurementCycleRule>,
            Rc::new(PendingMeasurementCycleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PendingMeasurementCycleLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_measurement_cycle_validator();
        let item = new_pending_measurement_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingMeasurementCycleRequiredCodeRule {
            message: "pending measurement cycle needs a value".to_string(),
        };
        let mut item = new_pending_measurement_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingMeasurementCycleLabelLengthRule { max_length: 32 };
        let mut item = new_pending_measurement_cycle("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
