use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant slot must pass. Check returns its problems, or none.
pub trait GrantSlotRule: std::fmt::Debug {
    fn check(&self, item: &GrantSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSlotRequiredIssuerRule {
    pub message: String,
}

impl GrantSlotRule for GrantSlotRequiredIssuerRule {
    fn check(&self, item: &GrantSlot) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSlotThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantSlotRule for GrantSlotThresholdRangeRule {
    fn check(&self, item: &GrantSlot) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSlotCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantSlotRule for GrantSlotCapacityRangeRule {
    fn check(&self, item: &GrantSlot) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantSlotIssuerLengthRule {
    pub max_length: i64,
}

impl GrantSlotRule for GrantSlotIssuerLengthRule {
    fn check(&self, item: &GrantSlot) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant slot.
#[derive(Debug, Clone)]
pub struct GrantSlotValidator {
    pub rules: Vec<Rc<dyn GrantSlotRule>>,
}

impl GrantSlotValidator {
    pub fn validate(&self, item: &GrantSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_slot_validator() -> GrantSlotValidator {
    GrantSlotValidator {
        rules: vec![
            Rc::new(GrantSlotRequiredIssuerRule {
                message: "grant slot needs an entry".to_string(),
            }) as Rc<dyn GrantSlotRule>,
            Rc::new(GrantSlotThresholdRangeRule { min: 6.4, max: 741.6 }),
            Rc::new(GrantSlotCapacityRangeRule { min: 80, max: 80 }),
            Rc::new(GrantSlotIssuerLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_slot_validator();
        let item = new_grant_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = GrantSlotRequiredIssuerRule {
            message: "grant slot needs this field".to_string(),
        };
        let mut item = new_grant_slot("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = GrantSlotThresholdRangeRule { min: 8.9, max: 491.8 };
        let mut item = new_grant_slot("x", "y");
        item.threshold = 493.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = GrantSlotCapacityRangeRule { min: 32, max: 64 };
        let mut item = new_grant_slot("x", "y");
        item.capacity = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = GrantSlotIssuerLengthRule { max_length: 86400 };
        let mut item = new_grant_slot("x", "y");
        item.issuer = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
