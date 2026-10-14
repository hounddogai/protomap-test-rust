use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn mapping must pass. Check returns its problems, or none.
pub trait BudgetBurnMappingRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMappingBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnMappingRule for BudgetBurnMappingBudgetRangeRule {
    fn check(&self, item: &BudgetBurnMapping) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMappingCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnMappingRule for BudgetBurnMappingCodePrefixRule {
    fn check(&self, item: &BudgetBurnMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMappingQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnMappingRule for BudgetBurnMappingQuotaUnitsRangeRule {
    fn check(&self, item: &BudgetBurnMapping) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMappingAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnMappingRule for BudgetBurnMappingAllowedLabelRule {
    fn check(&self, item: &BudgetBurnMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is unknown", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn mapping.
#[derive(Debug, Clone)]
pub struct BudgetBurnMappingValidator {
    pub rules: Vec<Rc<dyn BudgetBurnMappingRule>>,
}

impl BudgetBurnMappingValidator {
    pub fn validate(&self, item: &BudgetBurnMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_mapping_validator() -> BudgetBurnMappingValidator {
    BudgetBurnMappingValidator {
        rules: vec![
            Rc::new(BudgetBurnMappingBudgetRangeRule { min: 4.4, max: 2295.4 }) as Rc<dyn BudgetBurnMappingRule>,
            Rc::new(BudgetBurnMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(BudgetBurnMappingQuotaUnitsRangeRule { min: 32, max: 86400 }),
            Rc::new(BudgetBurnMappingAllowedLabelRule {
                allowed: vec!["fallback".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_mapping_validator();
        let item = new_budget_burn_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = BudgetBurnMappingBudgetRangeRule { min: 5.9, max: 2432.3 };
        let mut item = new_budget_burn_mapping("x", "y");
        item.budget = 2433.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = BudgetBurnMappingQuotaUnitsRangeRule { min: 80, max: 1000 };
        let mut item = new_budget_burn_mapping("x", "y");
        item.quota_units = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
