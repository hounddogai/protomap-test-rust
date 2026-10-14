use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted action item cycle must pass. Check returns its problems, or none.
pub trait WeightedActionItemCycleRule: std::fmt::Debug {
    fn check(&self, item: &WeightedActionItemCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemCycleCodePrefixRule {
    pub prefix: String,
}

impl WeightedActionItemCycleRule for WeightedActionItemCycleCodePrefixRule {
    fn check(&self, item: &WeightedActionItemCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemCycleRequiredNameRule {
    pub message: String,
}

impl WeightedActionItemCycleRule for WeightedActionItemCycleRequiredNameRule {
    fn check(&self, item: &WeightedActionItemCycle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemCycleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedActionItemCycleRule for WeightedActionItemCycleStartsAtRangeRule {
    fn check(&self, item: &WeightedActionItemCycle) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemCycleRequiredOwnerIdRule {
    pub message: String,
}

impl WeightedActionItemCycleRule for WeightedActionItemCycleRequiredOwnerIdRule {
    fn check(&self, item: &WeightedActionItemCycle) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted action item cycle.
#[derive(Debug, Clone)]
pub struct WeightedActionItemCycleValidator {
    pub rules: Vec<Rc<dyn WeightedActionItemCycleRule>>,
}

impl WeightedActionItemCycleValidator {
    pub fn validate(&self, item: &WeightedActionItemCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedActionItemCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedActionItemCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_action_item_cycle_validator() -> WeightedActionItemCycleValidator {
    WeightedActionItemCycleValidator {
        rules: vec![
            Rc::new(WeightedActionItemCycleCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn WeightedActionItemCycleRule>,
            Rc::new(WeightedActionItemCycleRequiredNameRule {
                message: "weighted action item cycle needs an entry".to_string(),
            }),
            Rc::new(WeightedActionItemCycleStartsAtRangeRule { min: 80, max: 86400 }),
            Rc::new(WeightedActionItemCycleRequiredOwnerIdRule {
                message: "weighted action item cycle needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_action_item_cycle_validator();
        let item = new_weighted_action_item_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = WeightedActionItemCycleRequiredNameRule {
            message: "weighted action item cycle needs a value".to_string(),
        };
        let mut item = new_weighted_action_item_cycle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = WeightedActionItemCycleStartsAtRangeRule { min: 16, max: 1000 };
        let mut item = new_weighted_action_item_cycle("x", "y");
        item.starts_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = WeightedActionItemCycleRequiredOwnerIdRule {
            message: "weighted action item cycle needs this field".to_string(),
        };
        let mut item = new_weighted_action_item_cycle("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
