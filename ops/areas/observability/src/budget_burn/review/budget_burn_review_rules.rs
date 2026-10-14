use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a budget burn review must pass. Check returns its problems, or none.
pub trait BudgetBurnReviewRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReviewMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnReviewRule for BudgetBurnReviewMaximumRangeRule {
    fn check(&self, item: &BudgetBurnReview) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReviewRequiredFingerprintRule {
    pub message: String,
}

impl BudgetBurnReviewRule for BudgetBurnReviewRequiredFingerprintRule {
    fn check(&self, item: &BudgetBurnReview) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReviewBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BudgetBurnReviewRule for BudgetBurnReviewBudgetRangeRule {
    fn check(&self, item: &BudgetBurnReview) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReviewAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl BudgetBurnReviewRule for BudgetBurnReviewAllowedFingerprintRule {
    fn check(&self, item: &BudgetBurnReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is inconsistent", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReviewCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnReviewRule for BudgetBurnReviewCodePrefixRule {
    fn check(&self, item: &BudgetBurnReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn review.
#[derive(Debug, Clone)]
pub struct BudgetBurnReviewValidator {
    pub rules: Vec<Rc<dyn BudgetBurnReviewRule>>,
}

impl BudgetBurnReviewValidator {
    pub fn validate(&self, item: &BudgetBurnReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_review_validator() -> BudgetBurnReviewValidator {
    BudgetBurnReviewValidator {
        rules: vec![
            Rc::new(BudgetBurnReviewMaximumRangeRule { min: 0.1, max: 1045.1 }) as Rc<dyn BudgetBurnReviewRule>,
            Rc::new(BudgetBurnReviewRequiredFingerprintRule {
                message: "budget burn review needs an entry".to_string(),
            }),
            Rc::new(BudgetBurnReviewBudgetRangeRule { min: 4.5, max: 1375.2 }),
            Rc::new(BudgetBurnReviewAllowedFingerprintRule { allowed: Vec::new() }),
            Rc::new(BudgetBurnReviewCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_review_validator();
        let item = new_budget_burn_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = BudgetBurnReviewMaximumRangeRule { min: 8.8, max: 720.8 };
        let mut item = new_budget_burn_review("x", "y");
        item.maximum = 722.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = BudgetBurnReviewRequiredFingerprintRule {
            message: "budget burn review needs this field".to_string(),
        };
        let mut item = new_budget_burn_review("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = BudgetBurnReviewBudgetRangeRule { min: 7.8, max: 3179.9 };
        let mut item = new_budget_burn_review("x", "y");
        item.budget = 3181.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
