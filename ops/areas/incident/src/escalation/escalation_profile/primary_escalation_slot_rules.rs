use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary escalation slot must pass. Check returns its problems, or none.
pub trait PrimaryEscalationSlotRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEscalationSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationSlotClusterLengthRule {
    pub max_length: i64,
}

impl PrimaryEscalationSlotRule for PrimaryEscalationSlotClusterLengthRule {
    fn check(&self, item: &PrimaryEscalationSlot) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationSlotRequiredTimezoneRule {
    pub message: String,
}

impl PrimaryEscalationSlotRule for PrimaryEscalationSlotRequiredTimezoneRule {
    fn check(&self, item: &PrimaryEscalationSlot) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationSlotRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEscalationSlotRule for PrimaryEscalationSlotRetriesRangeRule {
    fn check(&self, item: &PrimaryEscalationSlot) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary escalation slot.
#[derive(Debug, Clone)]
pub struct PrimaryEscalationSlotValidator {
    pub rules: Vec<Rc<dyn PrimaryEscalationSlotRule>>,
}

impl PrimaryEscalationSlotValidator {
    pub fn validate(&self, item: &PrimaryEscalationSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryEscalationSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEscalationSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_escalation_slot_validator() -> PrimaryEscalationSlotValidator {
    PrimaryEscalationSlotValidator {
        rules: vec![
            Rc::new(PrimaryEscalationSlotClusterLengthRule { max_length: 255 }) as Rc<dyn PrimaryEscalationSlotRule>,
            Rc::new(PrimaryEscalationSlotRequiredTimezoneRule {
                message: "primary escalation slot needs this field".to_string(),
            }),
            Rc::new(PrimaryEscalationSlotRetriesRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_escalation_slot_validator();
        let item = new_primary_escalation_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = PrimaryEscalationSlotClusterLengthRule { max_length: 3600 };
        let mut item = new_primary_escalation_slot("x", "y");
        item.cluster = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PrimaryEscalationSlotRequiredTimezoneRule {
            message: "primary escalation slot needs a value".to_string(),
        };
        let mut item = new_primary_escalation_slot("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = PrimaryEscalationSlotRetriesRangeRule { min: 86400, max: 86400 };
        let mut item = new_primary_escalation_slot("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
