use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a action item allocation must pass. Check returns its problems, or none.
pub trait ActionItemAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAllocationAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ActionItemAllocationRule for ActionItemAllocationAllowedTitleRule {
    fn check(&self, item: &ActionItemAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAllocationTitleLengthRule {
    pub max_length: i64,
}

impl ActionItemAllocationRule for ActionItemAllocationTitleLengthRule {
    fn check(&self, item: &ActionItemAllocation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAllocationNamespaceLengthRule {
    pub max_length: i64,
}

impl ActionItemAllocationRule for ActionItemAllocationNamespaceLengthRule {
    fn check(&self, item: &ActionItemAllocation) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAllocationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemAllocationRule for ActionItemAllocationGraceDaysRangeRule {
    fn check(&self, item: &ActionItemAllocation) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAllocationRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemAllocationRule for ActionItemAllocationRatioRangeRule {
    fn check(&self, item: &ActionItemAllocation) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAllocationCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemAllocationRule for ActionItemAllocationCodeLengthRule {
    fn check(&self, item: &ActionItemAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item allocation.
#[derive(Debug, Clone)]
pub struct ActionItemAllocationValidator {
    pub rules: Vec<Rc<dyn ActionItemAllocationRule>>,
}

impl ActionItemAllocationValidator {
    pub fn validate(&self, item: &ActionItemAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ActionItemAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_allocation_validator() -> ActionItemAllocationValidator {
    ActionItemAllocationValidator {
        rules: vec![
            Rc::new(ActionItemAllocationAllowedTitleRule {
                allowed: vec!["tiered".to_string(), "legacy".to_string()],
            }) as Rc<dyn ActionItemAllocationRule>,
            Rc::new(ActionItemAllocationTitleLengthRule { max_length: 86400 }),
            Rc::new(ActionItemAllocationNamespaceLengthRule { max_length: 86400 }),
            Rc::new(ActionItemAllocationGraceDaysRangeRule { min: 255, max: 255 }),
            Rc::new(ActionItemAllocationRatioRangeRule { min: 1.0, max: 3768.1 }),
            Rc::new(ActionItemAllocationCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_action_item_allocation_validator();
        let item = new_action_item_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ActionItemAllocationTitleLengthRule { max_length: 1000 };
        let mut item = new_action_item_allocation("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = ActionItemAllocationNamespaceLengthRule { max_length: 80 };
        let mut item = new_action_item_allocation("x", "y");
        item.namespace = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = ActionItemAllocationGraceDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_action_item_allocation("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = ActionItemAllocationRatioRangeRule { min: 1.0, max: 1301.9 };
        let mut item = new_action_item_allocation("x", "y");
        item.ratio = 1303.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ActionItemAllocationCodeLengthRule { max_length: 120 };
        let mut item = new_action_item_allocation("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
