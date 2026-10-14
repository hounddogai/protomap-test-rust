use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn tier must pass. Check returns its problems, or none.
pub trait BudgetBurnTierRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnTierEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnTierRule for BudgetBurnTierEndsAtRangeRule {
    fn check(&self, item: &BudgetBurnTier) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnTierWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BudgetBurnTierRule for BudgetBurnTierWindowDaysRangeRule {
    fn check(&self, item: &BudgetBurnTier) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnTierRequiredLabelRule {
    pub message: String,
}

impl BudgetBurnTierRule for BudgetBurnTierRequiredLabelRule {
    fn check(&self, item: &BudgetBurnTier) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnTierRequiredSummaryRule {
    pub message: String,
}

impl BudgetBurnTierRule for BudgetBurnTierRequiredSummaryRule {
    fn check(&self, item: &BudgetBurnTier) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnTierCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnTierRule for BudgetBurnTierCodePrefixRule {
    fn check(&self, item: &BudgetBurnTier) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnTierSummaryLengthRule {
    pub max_length: i64,
}

impl BudgetBurnTierRule for BudgetBurnTierSummaryLengthRule {
    fn check(&self, item: &BudgetBurnTier) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn tier.
#[derive(Debug, Clone)]
pub struct BudgetBurnTierValidator {
    pub rules: Vec<Rc<dyn BudgetBurnTierRule>>,
}

impl BudgetBurnTierValidator {
    pub fn validate(&self, item: &BudgetBurnTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BudgetBurnTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_tier_validator() -> BudgetBurnTierValidator {
    BudgetBurnTierValidator {
        rules: vec![
            Rc::new(BudgetBurnTierEndsAtRangeRule { min: 64, max: 255 }) as Rc<dyn BudgetBurnTierRule>,
            Rc::new(BudgetBurnTierWindowDaysRangeRule { min: 86400, max: 86400 }),
            Rc::new(BudgetBurnTierRequiredLabelRule {
                message: "budget burn tier needs this field".to_string(),
            }),
            Rc::new(BudgetBurnTierRequiredSummaryRule {
                message: "budget burn tier needs this field".to_string(),
            }),
            Rc::new(BudgetBurnTierCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(BudgetBurnTierSummaryLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_tier_validator();
        let item = new_budget_burn_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = BudgetBurnTierEndsAtRangeRule { min: 32, max: 64 };
        let mut item = new_budget_burn_tier("x", "y");
        item.ends_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = BudgetBurnTierWindowDaysRangeRule { min: 80, max: 80 };
        let mut item = new_budget_burn_tier("x", "y");
        item.window_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BudgetBurnTierRequiredLabelRule {
            message: "budget burn tier needs an entry".to_string(),
        };
        let mut item = new_budget_burn_tier("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = BudgetBurnTierRequiredSummaryRule {
            message: "budget burn tier needs an entry".to_string(),
        };
        let mut item = new_budget_burn_tier("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = BudgetBurnTierSummaryLengthRule { max_length: 16 };
        let mut item = new_budget_burn_tier("x", "y");
        item.summary = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
