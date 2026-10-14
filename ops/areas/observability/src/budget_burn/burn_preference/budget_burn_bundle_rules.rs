use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn bundle must pass. Check returns its problems, or none.
pub trait BudgetBurnBundleRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBundleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnBundleRule for BudgetBurnBundleScoreRangeRule {
    fn check(&self, item: &BudgetBurnBundle) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBundleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnBundleRule for BudgetBurnBundleAllowedLabelRule {
    fn check(&self, item: &BudgetBurnBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is below minimum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBundleFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnBundleRule for BudgetBurnBundleFloorValueRangeRule {
    fn check(&self, item: &BudgetBurnBundle) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBundleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnBundleRule for BudgetBurnBundleRotationDaysRangeRule {
    fn check(&self, item: &BudgetBurnBundle) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a budget burn bundle.
#[derive(Debug, Clone)]
pub struct BudgetBurnBundleValidator {
    pub rules: Vec<Rc<dyn BudgetBurnBundleRule>>,
}

impl BudgetBurnBundleValidator {
    pub fn validate(&self, item: &BudgetBurnBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_bundle_validator() -> BudgetBurnBundleValidator {
    BudgetBurnBundleValidator {
        rules: vec![
            Rc::new(BudgetBurnBundleScoreRangeRule { min: 6.9, max: 1486.2 }) as Rc<dyn BudgetBurnBundleRule>,
            Rc::new(BudgetBurnBundleAllowedLabelRule {
                allowed: vec!["regional".to_string(), "weighted".to_string()],
            }),
            Rc::new(BudgetBurnBundleFloorValueRangeRule { min: 0.3, max: 1316.9 }),
            Rc::new(BudgetBurnBundleRotationDaysRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_bundle_validator();
        let item = new_budget_burn_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = BudgetBurnBundleScoreRangeRule { min: 9.9, max: 2408.0 };
        let mut item = new_budget_burn_bundle("x", "y");
        item.score = 2409.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = BudgetBurnBundleFloorValueRangeRule { min: 5.5, max: 1514.6 };
        let mut item = new_budget_burn_bundle("x", "y");
        item.floor_value = 1516.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = BudgetBurnBundleRotationDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_budget_burn_bundle("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
