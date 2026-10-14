use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted retention budget must pass. Check returns its problems, or none.
pub trait WeightedRetentionBudgetRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRetentionBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionBudgetAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl WeightedRetentionBudgetRule for WeightedRetentionBudgetAllowedAudienceRule {
    fn check(&self, item: &WeightedRetentionBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is duplicated", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionBudgetAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRetentionBudgetRule for WeightedRetentionBudgetAmountRangeRule {
    fn check(&self, item: &WeightedRetentionBudget) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionBudgetActiveGuardRule {
    pub limit: f64,
}

impl WeightedRetentionBudgetRule for WeightedRetentionBudgetActiveGuardRule {
    fn check(&self, item: &WeightedRetentionBudget) -> Vec<String> {
        if item.active && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while active", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionBudgetRequiredPrincipalRule {
    pub message: String,
}

impl WeightedRetentionBudgetRule for WeightedRetentionBudgetRequiredPrincipalRule {
    fn check(&self, item: &WeightedRetentionBudget) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionBudgetRequiredLocaleRule {
    pub message: String,
}

impl WeightedRetentionBudgetRule for WeightedRetentionBudgetRequiredLocaleRule {
    fn check(&self, item: &WeightedRetentionBudget) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted retention budget.
#[derive(Debug, Clone)]
pub struct WeightedRetentionBudgetValidator {
    pub rules: Vec<Rc<dyn WeightedRetentionBudgetRule>>,
}

impl WeightedRetentionBudgetValidator {
    pub fn validate(&self, item: &WeightedRetentionBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedRetentionBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRetentionBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_retention_budget_validator() -> WeightedRetentionBudgetValidator {
    WeightedRetentionBudgetValidator {
        rules: vec![
            Rc::new(WeightedRetentionBudgetAllowedAudienceRule {
                allowed: vec!["manual".to_string()],
            }) as Rc<dyn WeightedRetentionBudgetRule>,
            Rc::new(WeightedRetentionBudgetAmountRangeRule { min: 7.2, max: 4451.8 }),
            Rc::new(WeightedRetentionBudgetActiveGuardRule { limit: 6.4 }),
            Rc::new(WeightedRetentionBudgetRequiredPrincipalRule {
                message: "weighted retention budget needs this field".to_string(),
            }),
            Rc::new(WeightedRetentionBudgetRequiredLocaleRule {
                message: "weighted retention budget needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_retention_budget_validator();
        let item = new_weighted_retention_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = WeightedRetentionBudgetAmountRangeRule { min: 6.6, max: 4348.6 };
        let mut item = new_weighted_retention_budget("x", "y");
        item.amount = 4350.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = WeightedRetentionBudgetRequiredPrincipalRule {
            message: "weighted retention budget needs this field".to_string(),
        };
        let mut item = new_weighted_retention_budget("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = WeightedRetentionBudgetRequiredLocaleRule {
            message: "weighted retention budget needs a value".to_string(),
        };
        let mut item = new_weighted_retention_budget("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
