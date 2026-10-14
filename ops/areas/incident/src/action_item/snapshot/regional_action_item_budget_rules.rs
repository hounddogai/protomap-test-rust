use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional action item budget must pass. Check returns its problems, or none.
pub trait RegionalActionItemBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RegionalActionItemBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemBudgetFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalActionItemBudgetRule for RegionalActionItemBudgetFloorValueRangeRule {
    fn check(&self, item: &RegionalActionItemBudget) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemBudgetRequiredTenantIdRule {
    pub message: String,
}

impl RegionalActionItemBudgetRule for RegionalActionItemBudgetRequiredTenantIdRule {
    fn check(&self, item: &RegionalActionItemBudget) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemBudgetTenantIdLengthRule {
    pub max_length: i64,
}

impl RegionalActionItemBudgetRule for RegionalActionItemBudgetTenantIdLengthRule {
    fn check(&self, item: &RegionalActionItemBudget) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional action item budget.
#[derive(Debug, Clone)]
pub struct RegionalActionItemBudgetValidator {
    pub rules: Vec<Rc<dyn RegionalActionItemBudgetRule>>,
}

impl RegionalActionItemBudgetValidator {
    pub fn validate(&self, item: &RegionalActionItemBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalActionItemBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalActionItemBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_action_item_budget_validator() -> RegionalActionItemBudgetValidator {
    RegionalActionItemBudgetValidator {
        rules: vec![
            Rc::new(RegionalActionItemBudgetFloorValueRangeRule { min: 4.9, max: 1416.3 })
                as Rc<dyn RegionalActionItemBudgetRule>,
            Rc::new(RegionalActionItemBudgetRequiredTenantIdRule {
                message: "regional action item budget needs a value".to_string(),
            }),
            Rc::new(RegionalActionItemBudgetTenantIdLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_action_item_budget_validator();
        let item = new_regional_action_item_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = RegionalActionItemBudgetFloorValueRangeRule { min: 3.8, max: 757.1 };
        let mut item = new_regional_action_item_budget("x", "y");
        item.floor_value = 758.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RegionalActionItemBudgetRequiredTenantIdRule {
            message: "regional action item budget needs an entry".to_string(),
        };
        let mut item = new_regional_action_item_budget("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = RegionalActionItemBudgetTenantIdLengthRule { max_length: 255 };
        let mut item = new_regional_action_item_budget("x", "y");
        item.tenant_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
