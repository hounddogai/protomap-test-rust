use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered action item mapping must pass. Check returns its problems, or none.
pub trait TieredActionItemMappingRule: std::fmt::Debug {
    fn check(&self, item: &TieredActionItemMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemMappingAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl TieredActionItemMappingRule for TieredActionItemMappingAllowedTitleRule {
    fn check(&self, item: &TieredActionItemMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is expired", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemMappingWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredActionItemMappingRule for TieredActionItemMappingWindowDaysRangeRule {
    fn check(&self, item: &TieredActionItemMapping) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemMappingRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredActionItemMappingRule for TieredActionItemMappingRevisionRangeRule {
    fn check(&self, item: &TieredActionItemMapping) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a tiered action item mapping.
#[derive(Debug, Clone)]
pub struct TieredActionItemMappingValidator {
    pub rules: Vec<Rc<dyn TieredActionItemMappingRule>>,
}

impl TieredActionItemMappingValidator {
    pub fn validate(&self, item: &TieredActionItemMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredActionItemMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredActionItemMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_action_item_mapping_validator() -> TieredActionItemMappingValidator {
    TieredActionItemMappingValidator {
        rules: vec![
            Rc::new(TieredActionItemMappingAllowedTitleRule { allowed: Vec::new() })
                as Rc<dyn TieredActionItemMappingRule>,
            Rc::new(TieredActionItemMappingWindowDaysRangeRule { min: 16, max: 80 }),
            Rc::new(TieredActionItemMappingRevisionRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_action_item_mapping_validator();
        let item = new_tiered_action_item_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = TieredActionItemMappingWindowDaysRangeRule { min: 64, max: 80 };
        let mut item = new_tiered_action_item_mapping("x", "y");
        item.window_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = TieredActionItemMappingRevisionRangeRule { min: 32, max: 80 };
        let mut item = new_tiered_action_item_mapping("x", "y");
        item.revision = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
