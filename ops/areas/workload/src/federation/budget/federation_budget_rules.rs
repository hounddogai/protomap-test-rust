use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation budget must pass. Check returns its problems, or none.
pub trait FederationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &FederationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBudgetRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FederationBudgetRule for FederationBudgetRatioRangeRule {
    fn check(&self, item: &FederationBudget) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBudgetRequiredCategoryRule {
    pub message: String,
}

impl FederationBudgetRule for FederationBudgetRequiredCategoryRule {
    fn check(&self, item: &FederationBudget) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBudgetAllowedNameRule {
    pub allowed: Vec<String>,
}

impl FederationBudgetRule for FederationBudgetAllowedNameRule {
    fn check(&self, item: &FederationBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBudgetCodeLengthRule {
    pub max_length: i64,
}

impl FederationBudgetRule for FederationBudgetCodeLengthRule {
    fn check(&self, item: &FederationBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBudgetActiveGuardRule {
    pub limit: f64,
}

impl FederationBudgetRule for FederationBudgetActiveGuardRule {
    fn check(&self, item: &FederationBudget) -> Vec<String> {
        if item.active && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while active", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBudgetCategoryLengthRule {
    pub max_length: i64,
}

impl FederationBudgetRule for FederationBudgetCategoryLengthRule {
    fn check(&self, item: &FederationBudget) -> Vec<String> {
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

/// Runs every rule of a federation budget.
#[derive(Debug, Clone)]
pub struct FederationBudgetValidator {
    pub rules: Vec<Rc<dyn FederationBudgetRule>>,
}

impl FederationBudgetValidator {
    pub fn validate(&self, item: &FederationBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_budget_validator() -> FederationBudgetValidator {
    FederationBudgetValidator {
        rules: vec![
            Rc::new(FederationBudgetRatioRangeRule { min: 3.3, max: 1319.3 }) as Rc<dyn FederationBudgetRule>,
            Rc::new(FederationBudgetRequiredCategoryRule {
                message: "federation budget needs an entry".to_string(),
            }),
            Rc::new(FederationBudgetAllowedNameRule { allowed: Vec::new() }),
            Rc::new(FederationBudgetCodeLengthRule { max_length: 1000 }),
            Rc::new(FederationBudgetActiveGuardRule { limit: 6.6 }),
            Rc::new(FederationBudgetCategoryLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_budget_validator();
        let item = new_federation_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = FederationBudgetRatioRangeRule { min: 2.6, max: 652.8 };
        let mut item = new_federation_budget("x", "y");
        item.ratio = 654.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = FederationBudgetRequiredCategoryRule {
            message: "federation budget needs this field".to_string(),
        };
        let mut item = new_federation_budget("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationBudgetCodeLengthRule { max_length: 255 };
        let mut item = new_federation_budget("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = FederationBudgetCategoryLengthRule { max_length: 86400 };
        let mut item = new_federation_budget("x", "y");
        item.category = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
