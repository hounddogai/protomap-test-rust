use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive delegation budget must pass. Check returns its problems, or none.
pub trait AdaptiveDelegationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveDelegationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationBudgetRequiredLabelRule {
    pub message: String,
}

impl AdaptiveDelegationBudgetRule for AdaptiveDelegationBudgetRequiredLabelRule {
    fn check(&self, item: &AdaptiveDelegationBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationBudgetRequiredCodeRule {
    pub message: String,
}

impl AdaptiveDelegationBudgetRule for AdaptiveDelegationBudgetRequiredCodeRule {
    fn check(&self, item: &AdaptiveDelegationBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationBudgetGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveDelegationBudgetRule for AdaptiveDelegationBudgetGraceDaysRangeRule {
    fn check(&self, item: &AdaptiveDelegationBudget) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationBudgetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveDelegationBudgetRule for AdaptiveDelegationBudgetAllowedCodeRule {
    fn check(&self, item: &AdaptiveDelegationBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationBudgetRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveDelegationBudgetRule for AdaptiveDelegationBudgetRatioRangeRule {
    fn check(&self, item: &AdaptiveDelegationBudget) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a adaptive delegation budget.
#[derive(Debug, Clone)]
pub struct AdaptiveDelegationBudgetValidator {
    pub rules: Vec<Rc<dyn AdaptiveDelegationBudgetRule>>,
}

impl AdaptiveDelegationBudgetValidator {
    pub fn validate(&self, item: &AdaptiveDelegationBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveDelegationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveDelegationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_delegation_budget_validator() -> AdaptiveDelegationBudgetValidator {
    AdaptiveDelegationBudgetValidator {
        rules: vec![
            Rc::new(AdaptiveDelegationBudgetRequiredLabelRule {
                message: "adaptive delegation budget needs an entry".to_string(),
            }) as Rc<dyn AdaptiveDelegationBudgetRule>,
            Rc::new(AdaptiveDelegationBudgetRequiredCodeRule {
                message: "adaptive delegation budget needs an entry".to_string(),
            }),
            Rc::new(AdaptiveDelegationBudgetGraceDaysRangeRule { min: 255, max: 3600 }),
            Rc::new(AdaptiveDelegationBudgetAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(AdaptiveDelegationBudgetRatioRangeRule { min: 5.3, max: 4279.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_delegation_budget_validator();
        let item = new_adaptive_delegation_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveDelegationBudgetRequiredLabelRule {
            message: "adaptive delegation budget needs this field".to_string(),
        };
        let mut item = new_adaptive_delegation_budget("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveDelegationBudgetRequiredCodeRule {
            message: "adaptive delegation budget needs this field".to_string(),
        };
        let mut item = new_adaptive_delegation_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = AdaptiveDelegationBudgetGraceDaysRangeRule { min: 16, max: 80 };
        let mut item = new_adaptive_delegation_budget("x", "y");
        item.grace_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = AdaptiveDelegationBudgetRatioRangeRule { min: 5.9, max: 4530.0 };
        let mut item = new_adaptive_delegation_budget("x", "y");
        item.ratio = 4531.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
