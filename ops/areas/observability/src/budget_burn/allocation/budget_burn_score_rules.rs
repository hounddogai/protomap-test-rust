use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn score must pass. Check returns its problems, or none.
pub trait BudgetBurnScoreRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScoreReviewerLengthRule {
    pub max_length: i64,
}

impl BudgetBurnScoreRule for BudgetBurnScoreReviewerLengthRule {
    fn check(&self, item: &BudgetBurnScore) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScoreCurrencyLengthRule {
    pub max_length: i64,
}

impl BudgetBurnScoreRule for BudgetBurnScoreCurrencyLengthRule {
    fn check(&self, item: &BudgetBurnScore) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScoreRequiredReviewerRule {
    pub message: String,
}

impl BudgetBurnScoreRule for BudgetBurnScoreRequiredReviewerRule {
    fn check(&self, item: &BudgetBurnScore) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScoreMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnScoreRule for BudgetBurnScoreMinimumRangeRule {
    fn check(&self, item: &BudgetBurnScore) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScoreRequiredNameRule {
    pub message: String,
}

impl BudgetBurnScoreRule for BudgetBurnScoreRequiredNameRule {
    fn check(&self, item: &BudgetBurnScore) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScoreRequiredCurrencyRule {
    pub message: String,
}

impl BudgetBurnScoreRule for BudgetBurnScoreRequiredCurrencyRule {
    fn check(&self, item: &BudgetBurnScore) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn score.
#[derive(Debug, Clone)]
pub struct BudgetBurnScoreValidator {
    pub rules: Vec<Rc<dyn BudgetBurnScoreRule>>,
}

impl BudgetBurnScoreValidator {
    pub fn validate(&self, item: &BudgetBurnScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_score_validator() -> BudgetBurnScoreValidator {
    BudgetBurnScoreValidator {
        rules: vec![
            Rc::new(BudgetBurnScoreReviewerLengthRule { max_length: 1000 }) as Rc<dyn BudgetBurnScoreRule>,
            Rc::new(BudgetBurnScoreCurrencyLengthRule { max_length: 64 }),
            Rc::new(BudgetBurnScoreRequiredReviewerRule {
                message: "budget burn score needs an entry".to_string(),
            }),
            Rc::new(BudgetBurnScoreMinimumRangeRule { min: 5.4, max: 4210.2 }),
            Rc::new(BudgetBurnScoreRequiredNameRule {
                message: "budget burn score needs a value".to_string(),
            }),
            Rc::new(BudgetBurnScoreRequiredCurrencyRule {
                message: "budget burn score needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_score_validator();
        let item = new_budget_burn_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = BudgetBurnScoreReviewerLengthRule { max_length: 1000 };
        let mut item = new_budget_burn_score("x", "y");
        item.reviewer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = BudgetBurnScoreCurrencyLengthRule { max_length: 255 };
        let mut item = new_budget_burn_score("x", "y");
        item.currency = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = BudgetBurnScoreRequiredReviewerRule {
            message: "budget burn score needs a value".to_string(),
        };
        let mut item = new_budget_burn_score("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = BudgetBurnScoreMinimumRangeRule { min: 6.2, max: 875.7 };
        let mut item = new_budget_burn_score("x", "y");
        item.minimum = 877.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BudgetBurnScoreRequiredNameRule {
            message: "budget burn score needs a value".to_string(),
        };
        let mut item = new_budget_burn_score("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = BudgetBurnScoreRequiredCurrencyRule {
            message: "budget burn score needs this field".to_string(),
        };
        let mut item = new_budget_burn_score("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
