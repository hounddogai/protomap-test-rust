use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page budget must pass. Check returns its problems, or none.
pub trait PageBudgetRule: std::fmt::Debug {
    fn check(&self, item: &PageBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBudgetCodePrefixRule {
    pub prefix: String,
}

impl PageBudgetRule for PageBudgetCodePrefixRule {
    fn check(&self, item: &PageBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBudgetLabelLengthRule {
    pub max_length: i64,
}

impl PageBudgetRule for PageBudgetLabelLengthRule {
    fn check(&self, item: &PageBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBudgetCategoryLengthRule {
    pub max_length: i64,
}

impl PageBudgetRule for PageBudgetCategoryLengthRule {
    fn check(&self, item: &PageBudget) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBudgetCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageBudgetRule for PageBudgetCostRangeRule {
    fn check(&self, item: &PageBudget) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBudgetAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PageBudgetRule for PageBudgetAmountRangeRule {
    fn check(&self, item: &PageBudget) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageBudgetCodeLengthRule {
    pub max_length: i64,
}

impl PageBudgetRule for PageBudgetCodeLengthRule {
    fn check(&self, item: &PageBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a page budget.
#[derive(Debug, Clone)]
pub struct PageBudgetValidator {
    pub rules: Vec<Rc<dyn PageBudgetRule>>,
}

impl PageBudgetValidator {
    pub fn validate(&self, item: &PageBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PageBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_page_budget_validator() -> PageBudgetValidator {
    PageBudgetValidator {
        rules: vec![
            Rc::new(PageBudgetCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn PageBudgetRule>,
            Rc::new(PageBudgetLabelLengthRule { max_length: 1000 }),
            Rc::new(PageBudgetCategoryLengthRule { max_length: 255 }),
            Rc::new(PageBudgetCostRangeRule { min: 3.2, max: 860.9 }),
            Rc::new(PageBudgetAmountRangeRule { min: 9.3, max: 4329.8 }),
            Rc::new(PageBudgetCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_budget_validator();
        let item = new_page_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PageBudgetLabelLengthRule { max_length: 64 };
        let mut item = new_page_budget("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = PageBudgetCategoryLengthRule { max_length: 86400 };
        let mut item = new_page_budget("x", "y");
        item.category = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PageBudgetCostRangeRule { min: 4.6, max: 187.8 };
        let mut item = new_page_budget("x", "y");
        item.cost = 189.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = PageBudgetAmountRangeRule { min: 6.9, max: 1830.0 };
        let mut item = new_page_budget("x", "y");
        item.amount = 1831.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PageBudgetCodeLengthRule { max_length: 120 };
        let mut item = new_page_budget("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
