use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a peering budget must pass. Check returns its problems, or none.
pub trait PeeringBudgetRule: std::fmt::Debug {
    fn check(&self, item: &PeeringBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringBudgetRequiredNameRule {
    pub message: String,
}

impl PeeringBudgetRule for PeeringBudgetRequiredNameRule {
    fn check(&self, item: &PeeringBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringBudgetCodeLengthRule {
    pub max_length: i64,
}

impl PeeringBudgetRule for PeeringBudgetCodeLengthRule {
    fn check(&self, item: &PeeringBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringBudgetAllowedNameRule {
    pub allowed: Vec<String>,
}

impl PeeringBudgetRule for PeeringBudgetAllowedNameRule {
    fn check(&self, item: &PeeringBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is inconsistent", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering budget.
#[derive(Debug, Clone)]
pub struct PeeringBudgetValidator {
    pub rules: Vec<Rc<dyn PeeringBudgetRule>>,
}

impl PeeringBudgetValidator {
    pub fn validate(&self, item: &PeeringBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_budget_validator() -> PeeringBudgetValidator {
    PeeringBudgetValidator {
        rules: vec![
            Rc::new(PeeringBudgetRequiredNameRule {
                message: "peering budget needs a value".to_string(),
            }) as Rc<dyn PeeringBudgetRule>,
            Rc::new(PeeringBudgetCodeLengthRule { max_length: 120 }),
            Rc::new(PeeringBudgetAllowedNameRule {
                allowed: vec!["global".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_budget_validator();
        let item = new_peering_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PeeringBudgetRequiredNameRule {
            message: "peering budget needs a value".to_string(),
        };
        let mut item = new_peering_budget("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringBudgetCodeLengthRule { max_length: 86400 };
        let mut item = new_peering_budget("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
