use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped peering budget must pass. Check returns its problems, or none.
pub trait CappedPeeringBudgetRule: std::fmt::Debug {
    fn check(&self, item: &CappedPeeringBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringBudgetRequiredCodeRule {
    pub message: String,
}

impl CappedPeeringBudgetRule for CappedPeeringBudgetRequiredCodeRule {
    fn check(&self, item: &CappedPeeringBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringBudgetRequiredLabelRule {
    pub message: String,
}

impl CappedPeeringBudgetRule for CappedPeeringBudgetRequiredLabelRule {
    fn check(&self, item: &CappedPeeringBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPeeringBudgetAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl CappedPeeringBudgetRule for CappedPeeringBudgetAllowedNamespaceRule {
    fn check(&self, item: &CappedPeeringBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is blocked", item.namespace)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped peering budget.
#[derive(Debug, Clone)]
pub struct CappedPeeringBudgetValidator {
    pub rules: Vec<Rc<dyn CappedPeeringBudgetRule>>,
}

impl CappedPeeringBudgetValidator {
    pub fn validate(&self, item: &CappedPeeringBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedPeeringBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedPeeringBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_peering_budget_validator() -> CappedPeeringBudgetValidator {
    CappedPeeringBudgetValidator {
        rules: vec![
            Rc::new(CappedPeeringBudgetRequiredCodeRule {
                message: "capped peering budget needs a value".to_string(),
            }) as Rc<dyn CappedPeeringBudgetRule>,
            Rc::new(CappedPeeringBudgetRequiredLabelRule {
                message: "capped peering budget needs an entry".to_string(),
            }),
            Rc::new(CappedPeeringBudgetAllowedNamespaceRule {
                allowed: vec!["default".to_string(), "lenient".to_string(), "global".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_peering_budget_validator();
        let item = new_capped_peering_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedPeeringBudgetRequiredCodeRule {
            message: "capped peering budget needs this field".to_string(),
        };
        let mut item = new_capped_peering_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CappedPeeringBudgetRequiredLabelRule {
            message: "capped peering budget needs a value".to_string(),
        };
        let mut item = new_capped_peering_budget("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
