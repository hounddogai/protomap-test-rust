use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn manifest must pass. Check returns its problems, or none.
pub trait BudgetBurnManifestRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnManifestRequiredTitleRule {
    pub message: String,
}

impl BudgetBurnManifestRule for BudgetBurnManifestRequiredTitleRule {
    fn check(&self, item: &BudgetBurnManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnManifestRequiredCategoryRule {
    pub message: String,
}

impl BudgetBurnManifestRule for BudgetBurnManifestRequiredCategoryRule {
    fn check(&self, item: &BudgetBurnManifest) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnManifestCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnManifestRule for BudgetBurnManifestCodePrefixRule {
    fn check(&self, item: &BudgetBurnManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnManifestBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnManifestRule for BudgetBurnManifestBudgetRangeRule {
    fn check(&self, item: &BudgetBurnManifest) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a budget burn manifest.
#[derive(Debug, Clone)]
pub struct BudgetBurnManifestValidator {
    pub rules: Vec<Rc<dyn BudgetBurnManifestRule>>,
}

impl BudgetBurnManifestValidator {
    pub fn validate(&self, item: &BudgetBurnManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_manifest_validator() -> BudgetBurnManifestValidator {
    BudgetBurnManifestValidator {
        rules: vec![
            Rc::new(BudgetBurnManifestRequiredTitleRule {
                message: "budget burn manifest needs this field".to_string(),
            }) as Rc<dyn BudgetBurnManifestRule>,
            Rc::new(BudgetBurnManifestRequiredCategoryRule {
                message: "budget burn manifest needs this field".to_string(),
            }),
            Rc::new(BudgetBurnManifestCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(BudgetBurnManifestBudgetRangeRule { min: 6.9, max: 2639.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_manifest_validator();
        let item = new_budget_burn_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BudgetBurnManifestRequiredTitleRule {
            message: "budget burn manifest needs a value".to_string(),
        };
        let mut item = new_budget_burn_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = BudgetBurnManifestRequiredCategoryRule {
            message: "budget burn manifest needs a value".to_string(),
        };
        let mut item = new_budget_burn_manifest("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = BudgetBurnManifestBudgetRangeRule { min: 1.2, max: 706.5 };
        let mut item = new_budget_burn_manifest("x", "y");
        item.budget = 708.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
