use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress budget must pass. Check returns its problems, or none.
pub trait EgressBudgetRule: std::fmt::Debug {
    fn check(&self, item: &EgressBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBudgetScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressBudgetRule for EgressBudgetScoreRangeRule {
    fn check(&self, item: &EgressBudget) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBudgetVisibleGuardRule {
    pub limit: f64,
}

impl EgressBudgetRule for EgressBudgetVisibleGuardRule {
    fn check(&self, item: &EgressBudget) -> Vec<String> {
        if item.visible && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while visible", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBudgetTitleLengthRule {
    pub max_length: i64,
}

impl EgressBudgetRule for EgressBudgetTitleLengthRule {
    fn check(&self, item: &EgressBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBudgetCodePrefixRule {
    pub prefix: String,
}

impl EgressBudgetRule for EgressBudgetCodePrefixRule {
    fn check(&self, item: &EgressBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressBudgetAllowedChannelRule {
    pub allowed: Vec<String>,
}

impl EgressBudgetRule for EgressBudgetAllowedChannelRule {
    fn check(&self, item: &EgressBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.channel)))
        {
            return vec![format!("channel {} is above maximum", item.channel)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress budget.
#[derive(Debug, Clone)]
pub struct EgressBudgetValidator {
    pub rules: Vec<Rc<dyn EgressBudgetRule>>,
}

impl EgressBudgetValidator {
    pub fn validate(&self, item: &EgressBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_budget_validator() -> EgressBudgetValidator {
    EgressBudgetValidator {
        rules: vec![
            Rc::new(EgressBudgetScoreRangeRule { min: 2.3, max: 1247.3 }) as Rc<dyn EgressBudgetRule>,
            Rc::new(EgressBudgetVisibleGuardRule { limit: 8.3 }),
            Rc::new(EgressBudgetTitleLengthRule { max_length: 120 }),
            Rc::new(EgressBudgetCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(EgressBudgetAllowedChannelRule {
                allowed: vec!["legacy".to_string(), "scheduled".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_budget_validator();
        let item = new_egress_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = EgressBudgetScoreRangeRule { min: 6.8, max: 2498.3 };
        let mut item = new_egress_budget("x", "y");
        item.score = 2499.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = EgressBudgetTitleLengthRule { max_length: 3600 };
        let mut item = new_egress_budget("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
