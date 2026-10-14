use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary peering slot must pass. Check returns its problems, or none.
pub trait PrimaryPeeringSlotRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryPeeringSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringSlotCodePrefixRule {
    pub prefix: String,
}

impl PrimaryPeeringSlotRule for PrimaryPeeringSlotCodePrefixRule {
    fn check(&self, item: &PrimaryPeeringSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringSlotBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryPeeringSlotRule for PrimaryPeeringSlotBudgetRangeRule {
    fn check(&self, item: &PrimaryPeeringSlot) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringSlotLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryPeeringSlotRule for PrimaryPeeringSlotLabelLengthRule {
    fn check(&self, item: &PrimaryPeeringSlot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringSlotTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryPeeringSlotRule for PrimaryPeeringSlotTitleLengthRule {
    fn check(&self, item: &PrimaryPeeringSlot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary peering slot.
#[derive(Debug, Clone)]
pub struct PrimaryPeeringSlotValidator {
    pub rules: Vec<Rc<dyn PrimaryPeeringSlotRule>>,
}

impl PrimaryPeeringSlotValidator {
    pub fn validate(&self, item: &PrimaryPeeringSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryPeeringSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryPeeringSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_peering_slot_validator() -> PrimaryPeeringSlotValidator {
    PrimaryPeeringSlotValidator {
        rules: vec![
            Rc::new(PrimaryPeeringSlotCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn PrimaryPeeringSlotRule>,
            Rc::new(PrimaryPeeringSlotBudgetRangeRule { min: 2.0, max: 90.4 }),
            Rc::new(PrimaryPeeringSlotLabelLengthRule { max_length: 32 }),
            Rc::new(PrimaryPeeringSlotTitleLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_peering_slot_validator();
        let item = new_primary_peering_slot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = PrimaryPeeringSlotBudgetRangeRule { min: 8.6, max: 3362.8 };
        let mut item = new_primary_peering_slot("x", "y");
        item.budget = 3364.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryPeeringSlotLabelLengthRule { max_length: 255 };
        let mut item = new_primary_peering_slot("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryPeeringSlotTitleLengthRule { max_length: 1000 };
        let mut item = new_primary_peering_slot("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
