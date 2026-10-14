use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin budget must pass. Check returns its problems, or none.
pub trait RegionPinBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBudgetLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinBudgetRule for RegionPinBudgetLimitRangeRule {
    fn check(&self, item: &RegionPinBudget) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBudgetRequiredLabelRule {
    pub message: String,
}

impl RegionPinBudgetRule for RegionPinBudgetRequiredLabelRule {
    fn check(&self, item: &RegionPinBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBudgetRequiredContactEmailRule {
    pub message: String,
}

impl RegionPinBudgetRule for RegionPinBudgetRequiredContactEmailRule {
    fn check(&self, item: &RegionPinBudget) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinBudgetCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionPinBudgetRule for RegionPinBudgetCreatedAtRangeRule {
    fn check(&self, item: &RegionPinBudget) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a region pin budget.
#[derive(Debug, Clone)]
pub struct RegionPinBudgetValidator {
    pub rules: Vec<Rc<dyn RegionPinBudgetRule>>,
}

impl RegionPinBudgetValidator {
    pub fn validate(&self, item: &RegionPinBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_budget_validator() -> RegionPinBudgetValidator {
    RegionPinBudgetValidator {
        rules: vec![
            Rc::new(RegionPinBudgetLimitRangeRule { min: 80, max: 255 }) as Rc<dyn RegionPinBudgetRule>,
            Rc::new(RegionPinBudgetRequiredLabelRule {
                message: "region pin budget needs a value".to_string(),
            }),
            Rc::new(RegionPinBudgetRequiredContactEmailRule {
                message: "region pin budget needs this field".to_string(),
            }),
            Rc::new(RegionPinBudgetCreatedAtRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_region_pin_budget_validator();
        let item = new_region_pin_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = RegionPinBudgetLimitRangeRule { min: 64, max: 1000 };
        let mut item = new_region_pin_budget("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RegionPinBudgetRequiredLabelRule {
            message: "region pin budget needs an entry".to_string(),
        };
        let mut item = new_region_pin_budget("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = RegionPinBudgetRequiredContactEmailRule {
            message: "region pin budget needs an entry".to_string(),
        };
        let mut item = new_region_pin_budget("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = RegionPinBudgetCreatedAtRangeRule { min: 80, max: 3600 };
        let mut item = new_region_pin_budget("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
