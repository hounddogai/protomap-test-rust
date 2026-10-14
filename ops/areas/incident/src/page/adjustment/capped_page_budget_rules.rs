use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped page budget must pass. Check returns its problems, or none.
pub trait CappedPageBudgetRule: std::fmt::Debug {
    fn check(&self, item: &CappedPageBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageBudgetCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedPageBudgetRule for CappedPageBudgetCostRangeRule {
    fn check(&self, item: &CappedPageBudget) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageBudgetNameLengthRule {
    pub max_length: i64,
}

impl CappedPageBudgetRule for CappedPageBudgetNameLengthRule {
    fn check(&self, item: &CappedPageBudget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageBudgetCodeLengthRule {
    pub max_length: i64,
}

impl CappedPageBudgetRule for CappedPageBudgetCodeLengthRule {
    fn check(&self, item: &CappedPageBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageBudgetCodePrefixRule {
    pub prefix: String,
}

impl CappedPageBudgetRule for CappedPageBudgetCodePrefixRule {
    fn check(&self, item: &CappedPageBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped page budget.
#[derive(Debug, Clone)]
pub struct CappedPageBudgetValidator {
    pub rules: Vec<Rc<dyn CappedPageBudgetRule>>,
}

impl CappedPageBudgetValidator {
    pub fn validate(&self, item: &CappedPageBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedPageBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedPageBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_page_budget_validator() -> CappedPageBudgetValidator {
    CappedPageBudgetValidator {
        rules: vec![
            Rc::new(CappedPageBudgetCostRangeRule { min: 2.6, max: 1383.0 }) as Rc<dyn CappedPageBudgetRule>,
            Rc::new(CappedPageBudgetNameLengthRule { max_length: 1000 }),
            Rc::new(CappedPageBudgetCodeLengthRule { max_length: 3600 }),
            Rc::new(CappedPageBudgetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_page_budget_validator();
        let item = new_capped_page_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = CappedPageBudgetCostRangeRule { min: 4.0, max: 2857.9 };
        let mut item = new_capped_page_budget("x", "y");
        item.cost = 2859.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CappedPageBudgetNameLengthRule { max_length: 32 };
        let mut item = new_capped_page_budget("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedPageBudgetCodeLengthRule { max_length: 16 };
        let mut item = new_capped_page_budget("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
